pub mod task_manager;
pub mod file_parser;
pub mod storage;
pub mod communication;
pub mod file_watcher;
pub mod error_macros;

pub use task_manager::{
    Task, TaskStatus, ProjectData, TaskSection, TaskIndex, ProjectMeta, TaskFile
};

pub use file_parser::{
    TaskParser, ParsedTaskLabel, ScanResult
};

pub use storage::{
    StorageManager, StorageInfo
};

pub use communication::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, JsonRpcHandler, JsonRpcServer, JsonRpcClient,
    ScanProjectParams, ScanProjectResult, GetTasksParams, UpdateTaskStatusParams,
    CreateTaskParams, DeleteTaskParams, FindTaskReferencesParams, TaskReference
};

pub use file_watcher::{
    FileWatcher, FileEvent, WatcherConfig, WatcherStats, EventDebouncer
};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub fn version_info() -> String {
    format!("{} v{}", NAME, VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("anchora"));
        assert!(info.contains("0.1.0"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(NAME, "anchora");
        assert_eq!(VERSION, "0.1.0");
    }
}