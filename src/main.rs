/*!
 * Anchora Task Manager Backend
 * 
 * Main entry point for the VSCode extension backend server.
 * Handles command-line arguments and starts the appropriate mode.
 */

use anchora::{
    JsonRpcServer, TaskManagerHandler, ScanProjectParams
};
use clap::{Arg, Command};
use std::path::PathBuf;

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

    let handler = TaskManagerHandler::new(workspace_path.clone())?;

    match mode.as_str() {
        "server" => {
            println!("Starting JSON-RPC server...");
            let server = JsonRpcServer::new(Box::new(handler));
            server.run_stdio().await?
        }
        "scan" => {
            println!("Scanning workspace for tasks...");
            let scan_params = ScanProjectParams {
                workspace_path: workspace_path.to_string_lossy().to_string(),
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
