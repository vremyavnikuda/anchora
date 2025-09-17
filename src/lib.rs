pub mod communication;
pub mod error_macros;
pub mod file_parser;
pub mod file_watcher;
pub mod handler;
pub mod search_engine;
pub mod statistics;
pub mod storage;
pub mod task_manager;
pub mod validation;

pub use task_manager::{
    Note, ProjectData, ProjectMeta, Task, TaskFile, TaskIndex, TaskSection, TaskStatus,
};

pub use file_parser::{ParsedTaskLabel, ScanResult, TaskParser};

pub use storage::{StorageInfo, StorageManager};

pub use communication::{
    BasicResponse,
    CheckConflictsParams,
    CreateNoteParams,
    CreateNoteResponse,
    CreateTaskParams,
    DeleteNoteParams,
    DeleteTaskParams,
    FindTaskReferencesParams,
    GenerateLinkParams,
    GenerateLinkResponse,
    GetFileDecorationsParams,
    GetFilteredTasksParams,
    GetStatisticsParams,
    GetSuggestionsParams,
    GetTaskOverviewParams,
    GetTasksParams,
    JsonRpcClient,
    JsonRpcError,
    JsonRpcHandler,
    JsonRpcRequest,
    JsonRpcResponse,
    JsonRpcServer,
    ScanProjectParams,
    ScanProjectResult,
    // New server-side operation parameters
    SearchTasksParams,
    TaskReference,
    UpdateTaskStatusParams,
    ValidateTaskParams,
};

pub use file_watcher::{EventDebouncer, FileEvent, FileWatcher, WatcherConfig, WatcherStats};

pub use search_engine::{
    MatchType, SearchEngine, SearchFilters, SearchQuery, SearchResult, Suggestion, SuggestionType,
    TaskSearchResult,
};

pub use statistics::{
    ActivityType, ChangeType, SectionStats, SectionSummary, StatisticsConfig, StatisticsManager,
    TaskActivity, TaskOverview, TaskStatistics, TaskUpdate,
};

pub use validation::{
    Conflict, ConflictCheck, ValidationConfig, ValidationEngine, ValidationError, ValidationParams,
    ValidationResult, ValidationWarning,
};

pub use handler::TaskManagerHandler;

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
