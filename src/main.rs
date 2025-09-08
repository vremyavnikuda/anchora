
use anchora::{file_parser, CreateTaskParams, FindTaskReferencesParams, GetTasksParams, JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse, JsonRpcServer, ScanProjectParams, ScanProjectResult, TaskParser, TaskReference, TaskStatus, UpdateTaskStatusParams};
use clap::{Arg, Command};
use std::path::PathBuf;
use std::sync::Arc;

struct TaskManagerHandler {
    storage: Arc<anchora::StorageManager>,
    parser: Arc<TaskParser>,
}

impl TaskManagerHandler {
    pub fn new(workspace_path: PathBuf) -> anyhow::Result<Self> {
        let storage = Arc::new(anchora::StorageManager::new(&workspace_path));
        let parser = Arc::new(TaskParser::new()?);
        
        Ok(Self { storage, parser })
    }
    async fn scan_project(&self, params: ScanProjectParams) -> anyhow::Result<ScanProjectResult> {
        let workspace_path = PathBuf::from(&params.workspace_path);
        let mut project_data = self.storage.load_project_data().await?;
        
        let mut scan_result = file_parser::ScanResult::new();
        let file_patterns = params.file_patterns.unwrap_or_else(|| {
            vec![
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
                "**/*.py".to_string(),
                "**/*.java".to_string(),
                "**/*.cpp".to_string(),
                "**/*.c".to_string(),
                "**/*.h".to_string(),
                "**/*.go".to_string(),
            ]
        });
        self.scan_directory_recursive(
            &workspace_path, 
            &workspace_path,
            &file_patterns, 
            &mut project_data, 
            &mut scan_result
        ).await?;
        project_data.rebuild_index();
        self.storage.save_project_data(&project_data).await?;

        Ok(ScanProjectResult {
            files_scanned: scan_result.files_scanned,
            tasks_found: scan_result.tasks_found,
            errors: scan_result.errors,
        })
    }
    async fn scan_directory_recursive(
        &self,
        current_path: &PathBuf,
        workspace_root: &PathBuf,
        file_patterns: &[String],
        project_data: &mut anchora::ProjectData,
        scan_result: &mut file_parser::ScanResult,
    ) -> anyhow::Result<()> {
        let ignored_dirs = [
            "target", "node_modules", ".git", ".vscode", ".anchora", 
            "dist", "build", "__pycache__", ".idea", "out"
        ];
        if let Ok(entries) = std::fs::read_dir(current_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if ignored_dirs.contains(&dir_name) {
                            continue;
                        }
                    }
                    Box::pin(self.scan_directory_recursive(
                        &path, 
                        workspace_root, 
                        file_patterns, 
                        project_data, 
                        scan_result
                    )).await?;
                } else if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if self.should_scan_file(file_name, file_patterns) {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let relative_path = path.strip_prefix(workspace_root)
                                    .unwrap_or(&path)
                                    .to_string_lossy()
                                    .replace('\\', "/");
                                match self.parser.scan_file(&relative_path, &content) {
                                    Ok(labels) => {
                                        scan_result.files_scanned += 1;
                                        scan_result.tasks_found += labels.len() as u32;
                                        if !labels.is_empty() {
                                            println!("Found {} tasks in file: {}", labels.len(), relative_path);
                                            for (line, label) in &labels {
                                                println!("  Line {}: {}:{} - {:?}", 
                                                    line, label.section, label.task_id, 
                                                    label.description.as_ref().unwrap_or(&"No description".to_string()));
                                            }
                                        }
                                        if let Err(e) = self.parser.update_project_from_labels(
                                            project_data,
                                            &relative_path,
                                            labels
                                        ) {
                                            scan_result.errors.push(format!("Error updating project data for {}: {}", relative_path, e));
                                        }
                                    }
                                    Err(e) => {
                                        scan_result.errors.push(format!("Error scanning file {}: {}", relative_path, e));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn should_scan_file(&self, file_name: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if pattern.starts_with("**/*.") {
                let extension = &pattern[5..];
                if file_name.ends_with(&format!(".{}", extension)) {
                    return true;
                }
            } else if pattern.starts_with("*.") {
                let extension = &pattern[2..];
                if file_name.ends_with(&format!(".{}", extension)) {
                    return true;
                }
            }
        }
        false
    }
    async fn get_tasks(&self, _params: Option<GetTasksParams>) -> anyhow::Result<serde_json::Value> {
        let project_data = self.storage.load_project_data().await?;
        Ok(serde_json::to_value(&project_data.sections)?)
    }
    async fn create_task(&self, params: CreateTaskParams) -> anyhow::Result<serde_json::Value> {
        let mut project_data = self.storage.load_project_data().await?;
        project_data.add_task(
            &params.section,
            &params.task_id,
            params.title,
            params.description
        )?;
        self.storage.save_project_data(&project_data).await?;
        Ok(serde_json::json!({
            "success": true,
            "message": format!("Task {}:{} created successfully", params.section, params.task_id)
        }))
    }
    async fn update_task_status(&self, params: UpdateTaskStatusParams) -> anyhow::Result<serde_json::Value> {
        let mut project_data = self.storage.load_project_data().await?;
        let status = match params.status.to_lowercase().as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" | "inprogress" => TaskStatus::InProgress,
            "done" | "completed" => TaskStatus::Done,
            "blocked" => TaskStatus::Blocked,
            _ => return Err(anyhow::anyhow!("Invalid status: {}", params.status)),
        };
        project_data.update_task_status(&params.section, &params.task_id, status)?;
        self.storage.save_project_data(&project_data).await?;
        Ok(serde_json::json!({
            "success": true,
            "message": format!("Task {}:{} status updated to {}", params.section, params.task_id, params.status)
        }))
    }
    async fn find_task_references(&self, params: FindTaskReferencesParams) -> anyhow::Result<Vec<TaskReference>> {
        let project_data = self.storage.load_project_data().await?;
        if let Some(task) = project_data.get_task(&params.section, &params.task_id) {
            let mut references = Vec::new();
            for (file_path, task_file) in &task.files {
                for &line in &task_file.lines {
                    references.push(TaskReference {
                        file_path: file_path.clone(),
                        line,
                        note: task_file.notes.get(&line).cloned(),
                    });
                }
            }
            Ok(references)
        } else {
            Err(anyhow::anyhow!("Task not found: {}:{}", params.section, params.task_id))
        }
    }
}

impl JsonRpcHandler for TaskManagerHandler {
    fn handle_request(&self, request: JsonRpcRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = JsonRpcResponse> + Send + '_>> {
        Box::pin(async move {
            match request.method.as_str() {
            "scan_project" => {
                match request.params {
                    Some(params) => {
                        match serde_json::from_value::<ScanProjectParams>(params) {
                            Ok(scan_params) => {
                                match self.scan_project(scan_params).await {
                                    Ok(result) => JsonRpcServer::success_response(
                                        request.id,
                                        serde_json::to_value(result).unwrap_or_default()
                                    ),
                                    Err(e) => JsonRpcServer::error_response(
                                        request.id,
                                        JsonRpcError::custom(-1, format!("Scan project failed: {}", e), None)
                                    )
                                }
                            }
                            Err(e) => JsonRpcServer::error_response(
                                request.id,
                                JsonRpcError::custom(-1, format!("Invalid scan_project params: {}", e), None)
                            )
                        }
                    }
                    None => JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::invalid_params()
                    )
                }
            }
            "get_tasks" => {
                let params = request.params.and_then(|p| serde_json::from_value(p).ok());
                match self.get_tasks(params).await {
                    Ok(result) => JsonRpcServer::success_response(request.id, result),
                    Err(e) => JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::custom(-1, format!("Get tasks failed: {}", e), None)
                    )
                }
            }
            "create_task" => {
                match request.params {
                    Some(params) => {
                        match serde_json::from_value::<CreateTaskParams>(params) {
                            Ok(create_params) => {
                                match self.create_task(create_params).await {
                                    Ok(result) => JsonRpcServer::success_response(request.id, result),
                                    Err(e) => JsonRpcServer::error_response(
                                        request.id,
                                        JsonRpcError::custom(-1, format!("Create task failed: {}", e), None)
                                    )
                                }
                            }
                            Err(e) => JsonRpcServer::error_response(
                                request.id,
                                JsonRpcError::custom(-1, format!("Invalid create_task params: {}", e), None)
                            )
                        }
                    }
                    None => JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::invalid_params()
                    )
                }
            }
            "update_task_status" => {
                match request.params {
                    Some(params) => {
                        match serde_json::from_value::<UpdateTaskStatusParams>(params) {
                            Ok(update_params) => {
                                match self.update_task_status(update_params).await {
                                    Ok(result) => JsonRpcServer::success_response(request.id, result),
                                    Err(e) => JsonRpcServer::error_response(
                                        request.id,
                                        JsonRpcError::custom(-1, format!("Update task status failed: {}", e), None)
                                    )
                                }
                            }
                            Err(e) => JsonRpcServer::error_response(
                                request.id,
                                JsonRpcError::custom(-1, format!("Invalid update_task_status params: {}", e), None)
                            )
                        }
                    }
                    None => JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::invalid_params()
                    )
                }
            }
            "find_task_references" => {
                match request.params {
                    Some(params) => {
                        match serde_json::from_value::<FindTaskReferencesParams>(params) {
                            Ok(find_params) => {
                                match self.find_task_references(find_params).await {
                                    Ok(result) => JsonRpcServer::success_response(
                                        request.id,
                                        serde_json::to_value(result).unwrap_or_default()
                                    ),
                                    Err(e) => JsonRpcServer::error_response(
                                        request.id,
                                        JsonRpcError::custom(-1, format!("Find task references failed: {}", e), None)
                                    )
                                }
                            }
                            Err(e) => JsonRpcServer::error_response(
                                request.id,
                                JsonRpcError::custom(-1, format!("Invalid find_task_references params: {}", e), None)
                            )
                        }
                    }
                    None => JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::invalid_params()
                    )
                }
            }
            _ => JsonRpcServer::error_response(
                request.id,
                JsonRpcError::method_not_found()
            )
        }
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("anchora")
        .version("0.1.0")
        .about("Task Manager Backend for VSCode Extension")
        .arg(
            Arg::new("workspace")
                .short('w')
                .long("workspace")
                .value_name("PATH")
                .help("Workspace directory path")
                .required(true)
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Execution mode: server, scan")
                .default_value("server")
        )
        .get_matches();

    let workspace_path = PathBuf::from(
        matches.get_one::<String>("workspace")
            .expect("Workspace path is required")
    );
    let mode = matches.get_one::<String>("mode").unwrap();
    println!("Anchora Task Manager Backend v0.1.0");
    println!("Workspace: {:?}", workspace_path);
    println!("Mode: {}", mode);
    let handler = TaskManagerHandler::new(workspace_path)?;
    match mode.as_str() {
        "server" => {
            println!("Starting JSON-RPC server...");
            let server = JsonRpcServer::new(Box::new(handler));
            server.run_stdio().await?
        }
        "scan" => {
            println!("Scanning workspace for tasks...");
            let scan_params = ScanProjectParams {
                workspace_path: matches.get_one::<String>("workspace").unwrap().clone(),
                file_patterns: None,
            };
            let result = handler.scan_project(scan_params).await?;
            println!("Scan completed:");
            println!("  Files scanned: {}", result.files_scanned);
            println!("  Tasks found: {}", result.tasks_found);
            if !result.errors.is_empty() {
                println!("  Errors:");
                for error in &result.errors {
                    println!("    - {}", error);
                }
            }
        }
        _ => {
            eprintln!("Unknown mode: {}. Use 'server' or 'scan'", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
