/*!
 * Task Manager Handler Module
 * 
 * Contains the main business logic for handling JSON-RPC requests
 * and managing task operations.
 */

use crate::{
    file_parser, CreateTaskParams, DeleteTaskParams, FindTaskReferencesParams, GetTasksParams,
    JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse, JsonRpcServer, ScanProjectParams,
    ScanProjectResult, TaskParser, TaskReference, TaskStatus, UpdateTaskStatusParams, CreateNoteParams,
    CreateNoteResponse, GenerateLinkParams, DeleteNoteParams, GenerateLinkResponse, BasicResponse, Note,
    SearchEngine, SearchQuery, StatisticsManager, ValidationEngine,
    SearchTasksParams, ValidateTaskParams,
    GetSuggestionsParams, CheckConflictsParams, ValidationParams
};
use crate::{handle_jsonrpc_method, handle_simple_method, handle_parameterized_method};
use std::path::PathBuf;
use std::sync::Arc;
use chrono;

pub struct TaskManagerHandler {
    storage: Arc<crate::StorageManager>,
    parser: Arc<TaskParser>,
    search_engine: Arc<SearchEngine>,
    statistics_manager: Arc<StatisticsManager>,
    validation_engine: Arc<ValidationEngine>,
}

impl TaskManagerHandler {
    pub fn new(workspace_path: PathBuf) -> anyhow::Result<Self> {
        let storage = Arc::new(crate::StorageManager::new(&workspace_path));
        let parser = Arc::new(TaskParser::new()?);
        let search_engine = Arc::new(SearchEngine::new());
        let statistics_manager = Arc::new(StatisticsManager::new(None));
        let validation_engine = Arc::new(ValidationEngine::new(None));
        
        Ok(Self { 
            storage, 
            parser,
            search_engine,
            statistics_manager,
            validation_engine,
        })
    }

    pub async fn scan_project(&self, params: ScanProjectParams) -> anyhow::Result<ScanProjectResult> {
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
                "**/*.hpp".to_string(),
                "**/*.cc".to_string(),
                "**/*.cxx".to_string(),
                "**/*.go".to_string(),
                "**/*.php".to_string(),
                "**/*.rb".to_string(),
                "**/*.swift".to_string(),
                "**/*.kt".to_string(),
                "**/*.scala".to_string(),
                "**/*.cs".to_string(),
                "**/*.fs".to_string(),
                "**/*.vb".to_string(),
                "**/*.dart".to_string(),
                "**/*.elm".to_string(),
                "**/*.hs".to_string(),
                "**/*.ml".to_string(),
                "**/*.clj".to_string(),
                "**/*.ex".to_string(),
                "**/*.exs".to_string(),
                "**/*.erl".to_string(),
                "**/*.jl".to_string(),
                "**/*.r".to_string(),
                "**/*.m".to_string(),
                "**/*.mm".to_string(),
                "**/*.pl".to_string(),
                "**/*.pm".to_string(),
                "**/*.lua".to_string(),
                "**/*.sh".to_string(),
                "**/*.ps1".to_string(),
                "**/*.bat".to_string(),
                "**/*.cmd".to_string(),
                "**/*.jsx".to_string(),
                "**/*.tsx".to_string(),
                "**/*.vue".to_string(),
                "**/*.svelte".to_string(),
                "**/*.sql".to_string(),
                "**/*.yaml".to_string(),
                "**/*.yml".to_string(),
                "**/*.toml".to_string(),
                "**/*.ini".to_string(),
                "**/*.cfg".to_string(),
                "**/*.conf".to_string(),
                "**/*.dockerfile".to_string(),
                "**/*.tf".to_string(),
                "**/*.hcl".to_string(),
                "**/*.json".to_string(),
                "**/*.xml".to_string(),
                "**/*.html".to_string(),
                "**/*.css".to_string(),
                "**/*.scss".to_string(),
                "**/*.sass".to_string(),
                "**/*.less".to_string(),
                "**/*.md".to_string(),
                "**/*.rst".to_string(),
                "**/*.tex".to_string(),
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
        project_data: &mut crate::ProjectData,
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
        Ok(serde_json::to_value(&project_data)?)
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

    async fn delete_task(&self, params: DeleteTaskParams) -> anyhow::Result<serde_json::Value> {
        let mut project_data = self.storage.load_project_data().await?;
        project_data.delete_task(&params.section, &params.task_id)?;
        self.storage.save_project_data(&project_data).await?;
        Ok(serde_json::json!({
            "success": true,
            "message": format!("Task {}:{} deleted successfully", params.section, params.task_id)
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

    async fn create_note(&self, params: CreateNoteParams) -> anyhow::Result<CreateNoteResponse> {
        let mut project_data = self.storage.load_project_data().await?;
        let suggested_status = if let Some(status_str) = params.suggested_status {
            match status_str.to_lowercase().as_str() {
                "todo" => Some(TaskStatus::Todo),
                "in_progress" | "inprogress" => Some(TaskStatus::InProgress),
                "done" | "completed" => Some(TaskStatus::Done),
                "blocked" => Some(TaskStatus::Blocked),
                _ => return Err(anyhow::anyhow!("Invalid status: {}", status_str)),
            }
        } else {
            None
        };

        let note_id = project_data.add_note(
            params.title.clone(),
            params.content,
            params.section,
            params.suggested_task_id,
            suggested_status,
        )?;

        self.storage.save_project_data(&project_data).await?;
        Ok(CreateNoteResponse {
            success: true,
            message: format!("Note '{}' created successfully", params.title),
            note_id,
        })
    }

    async fn get_notes(&self) -> anyhow::Result<Vec<Note>> {
        let project_data = self.storage.load_project_data().await?;
        Ok(project_data.get_all_notes().into_iter().cloned().collect())
    }

    async fn generate_task_link(&self, note_id: String) -> anyhow::Result<GenerateLinkResponse> {
        let mut project_data = self.storage.load_project_data().await?;
        let link = project_data.generate_note_link(&note_id)?;
        self.storage.save_project_data(&project_data).await?;
        Ok(GenerateLinkResponse {
            success: true,
            link,
        })
    }

    async fn delete_note(&self, note_id: String) -> anyhow::Result<BasicResponse> {
        let mut project_data = self.storage.load_project_data().await?;
        project_data.delete_note(&note_id)?;
        self.storage.save_project_data(&project_data).await?;
        Ok(BasicResponse {
            success: true,
            message: "Note deleted successfully".to_string(),
        })
    }

    // New server-side operation implementations

    async fn search_tasks(&self, params: SearchTasksParams) -> anyhow::Result<serde_json::Value> {
        // Load current project data and update search index
        let project_data = self.storage.load_project_data().await?;
        self.search_engine.index_project(&project_data)?;

        // Convert params to search query
        let search_query = SearchQuery {
            query: params.query,
            filters: params.filters.and_then(|f| serde_json::from_value(f).ok()),
            limit: params.limit,
            offset: params.offset,
        };

        // Perform search
        let result = self.search_engine.search(&search_query)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn get_statistics(&self) -> anyhow::Result<serde_json::Value> {
        let project_data = self.storage.load_project_data().await?;
        
        // Update contexts
        self.statistics_manager.get_statistics(&project_data).map(|stats| serde_json::to_value(stats).unwrap_or(serde_json::Value::Null))
    }

    async fn get_task_overview(&self) -> anyhow::Result<serde_json::Value> {
        let project_data = self.storage.load_project_data().await?;
        
        // Get basic overview data
        let overview = self.statistics_manager.get_overview(&project_data)?;
        
        // Get recent activity
        let recent_activity = self.statistics_manager.get_recent_activity()?;
        
        // Build sections with actual task data
        let mut sections_with_tasks = Vec::new();
        for section_summary in &overview.sections {
            let mut section_tasks = Vec::new();
            
            // Get actual tasks from the section
            if let Some(section_data) = project_data.sections.get(&section_summary.name) {
                for (task_id, task) in section_data {
                    let task_info = serde_json::json!({
                        "id": task_id,
                        "title": task.title,
                        "description": task.description,
                        "status": task.status,
                        "created": task.created.to_rfc3339(),
                        "updated": task.updated.to_rfc3339(),
                        "fileCount": task.files.len()
                    });
                    section_tasks.push(task_info);
                }
            }
            
            let section_with_tasks = serde_json::json!({
                "name": section_summary.name,
                "total_tasks": section_summary.total_tasks,
                "completion_percentage": section_summary.completion_percentage,
                "active_tasks": section_summary.active_tasks,
                "blocked_tasks": section_summary.blocked_tasks,
                "recent_changes": section_summary.recent_changes,
                "tasks": section_tasks
            });
            sections_with_tasks.push(section_with_tasks);
        }
        
        // Create TaskStatistics structure that matches frontend expectations
        let task_statistics = serde_json::json!({
            "total_tasks": overview.total_tasks,
            "by_status": {
                "todo": overview.total_tasks - overview.completed_tasks - overview.in_progress_tasks - overview.blocked_tasks,
                "in_progress": overview.in_progress_tasks,
                "done": overview.completed_tasks,
                "blocked": overview.blocked_tasks
            },
            "by_section": {}, // TODO: implement section-wise stats
            "recent_updates": [], // TODO: implement recent updates
            "performance_metrics": {
                "calculation_time_ms": 0,
                "cache_hit_rate": 0.0,
                "last_cache_update": chrono::Utc::now().to_rfc3339(),
                "total_calculations": 0
            },
            "last_calculated": chrono::Utc::now().to_rfc3339(),
            "trends": {
                "daily_completions": [],
                "section_velocity": {},
                "status_transitions": {},
                "productivity_score": 75.0
            }
        });
        
        // Create the complete TaskOverview structure expected by frontend
        let complete_overview = serde_json::json!({
            "sections": sections_with_tasks,
            "statistics": task_statistics,
            "recent_activity": recent_activity,
            "recommendations": []
        });
        
        Ok(complete_overview)
    }

    async fn validate_task_input(&self, params: ValidateTaskParams) -> anyhow::Result<serde_json::Value> {
        let project_data = self.storage.load_project_data().await?;
        self.validation_engine.update_context(project_data)?;
        let validation_params = ValidationParams {
            section: params.section,
            task_id: params.task_id,
            title: params.title,
            description: params.description,
            check_duplicates: params.check_duplicates,
            suggest_alternatives: params.suggest_alternatives,
        };
        
        let result = self.validation_engine.validate_task_creation(&validation_params)?;
        Ok(serde_json::to_value(result)?)
    }

    async fn get_suggestions(&self, params: GetSuggestionsParams) -> anyhow::Result<serde_json::Value> {
        let suggestions = self.search_engine.get_suggestions(&params.partial_query)?;
        Ok(serde_json::to_value(suggestions)?)
    }

    async fn check_task_conflicts(&self, params: CheckConflictsParams) -> anyhow::Result<serde_json::Value> {
        let project_data = self.storage.load_project_data().await?;
        self.validation_engine.update_context(project_data)?;
        let result = self.validation_engine.check_task_conflicts(&params.section, &params.task_id)?;
        Ok(serde_json::to_value(result)?)
    }
}

impl JsonRpcHandler for TaskManagerHandler {
    fn handle_request(&self, request: JsonRpcRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = JsonRpcResponse> + Send + '_>> {
        Box::pin(async move {
            match request.method.as_str() {
                "scan_project" => {
                    handle_parameterized_method!(
                        request,
                        ScanProjectParams,
                        "scan_project",
                        "Scan project for tasks",
                        |params| self.scan_project(params)
                    )
                }
                "get_tasks" => {
                    let params = request.params.and_then(|p| serde_json::from_value(p).ok());
                    handle_simple_method!(
                        request.id,
                        "get_tasks",
                        "Retrieve project tasks",
                        self.get_tasks(params)
                    )
                }
                "create_task" => {
                    handle_parameterized_method!(
                        request,
                        CreateTaskParams,
                        "create_task",
                        "Create new task",
                        |params| self.create_task(params)
                    )
                }
                "update_task_status" => {
                    handle_parameterized_method!(
                        request,
                        UpdateTaskStatusParams,
                        "update_task_status",
                        "Update task status",
                        |params| self.update_task_status(params)
                    )
                }
                "delete_task" => {
                    handle_parameterized_method!(
                        request,
                        DeleteTaskParams,
                        "delete_task",
                        "Delete task",
                        |params| self.delete_task(params)
                    )
                }
                "find_task_references" => {
                    handle_parameterized_method!(
                        request,
                        FindTaskReferencesParams,
                        "find_task_references",
                        "Find task references",
                        |params| async {
                            self.find_task_references(params).await
                        }
                    )
                }
                "create_note" => {
                    handle_parameterized_method!(
                        request,
                        CreateNoteParams,
                        "create_note",
                        "Create new note",
                        |params| self.create_note(params)
                    )
                }
                "get_notes" => {
                    handle_simple_method!(
                        request.id,
                        "get_notes",
                        "Retrieve all notes",
                        self.get_notes()
                    )
                }
                "generate_task_link" => {
                    handle_parameterized_method!(
                        request,
                        GenerateLinkParams,
                        "generate_task_link",
                        "Generate task link for note",
                        |params| self.generate_task_link(params.note_id)
                    )
                }
                "delete_note" => {
                    handle_parameterized_method!(
                        request,
                        DeleteNoteParams,
                        "delete_note",
                        "Delete note",
                        |params| self.delete_note(params.note_id)
                    )
                }
                "search_tasks" => {
                    handle_parameterized_method!(
                        request,
                        SearchTasksParams,
                        "search_tasks",
                        "Search tasks with indexing",
                        |params| self.search_tasks(params)
                    )
                }
                "get_statistics" => {
                    handle_simple_method!(
                        request.id,
                        "get_statistics",
                        "Get task statistics",
                        self.get_statistics()
                    )
                }
                "get_task_overview" => {
                    handle_simple_method!(
                        request.id,
                        "get_task_overview",
                        "Get task overview",
                        self.get_task_overview()
                    )
                }
                "validate_task_input" => {
                    handle_parameterized_method!(
                        request,
                        ValidateTaskParams,
                        "validate_task_input",
                        "Validate task input",
                        |params| self.validate_task_input(params)
                    )
                }
                "get_suggestions" => {
                    handle_parameterized_method!(
                        request,
                        GetSuggestionsParams,
                        "get_suggestions",
                        "Get task suggestions",
                        |params| self.get_suggestions(params)
                    )
                }
                "check_task_conflicts" => {
                    handle_parameterized_method!(
                        request,
                        CheckConflictsParams,
                        "check_task_conflicts",
                        "Check task conflicts",
                        |params| self.check_task_conflicts(params)
                    )
                }
                _ => {
                    eprintln!("[ERROR] Unknown method: {}", request.method);
                    JsonRpcServer::error_response(
                        request.id,
                        JsonRpcError::method_not_found()
                    )
                }
            }
        })
    }
}