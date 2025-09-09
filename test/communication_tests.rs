use anchora::communication::*;
use serde_json::{json, Value};

#[test]
fn test_jsonrpc_request_creation() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "test_method".to_string(),
        params: Some(json!({
            "param1": "value1",
            "param2": 42
        })),
        id: Some(Value::Number(serde_json::Number::from(1))),
    };
    
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test_method");
    assert!(request.params.is_some());
    assert!(request.id.is_some());
}

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "scan_project".to_string(),
        params: Some(json!({
            "workspace_path": "/path/to/project",
            "file_patterns": ["**/*.rs", "**/*.ts"]
        })),
        id: Some(Value::Number(serde_json::Number::from(123))),
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.jsonrpc, request.jsonrpc);
    assert_eq!(deserialized.method, request.method);
    assert_eq!(deserialized.id, request.id);
}

#[test]
fn test_jsonrpc_response_success() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "files_scanned": 10,
            "tasks_found": 5
        })),
        error: None,
        id: Some(Value::Number(serde_json::Number::from(1))),
    };
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_jsonrpc_response_error() {
    let error = JsonRpcError::method_not_found();
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(error.clone()),
        id: Some(Value::Number(serde_json::Number::from(1))),
    };
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, error.code);
}

#[test]
fn test_jsonrpc_error_types() {
    let parse_error = JsonRpcError::parse_error();
    assert_eq!(parse_error.code, -32700);
    assert_eq!(parse_error.message, "Parse error");
    
    let invalid_request = JsonRpcError::invalid_request();
    assert_eq!(invalid_request.code, -32600);
    assert_eq!(invalid_request.message, "Invalid Request");
    
    let method_not_found = JsonRpcError::method_not_found();
    assert_eq!(method_not_found.code, -32601);
    assert_eq!(method_not_found.message, "Method not found");
    
    let invalid_params = JsonRpcError::invalid_params();
    assert_eq!(invalid_params.code, -32602);
    assert_eq!(invalid_params.message, "Invalid params");
    
    let internal_error = JsonRpcError::internal_error();
    assert_eq!(internal_error.code, -32603);
    assert_eq!(internal_error.message, "Internal error");
}

#[test]
fn test_jsonrpc_custom_error() {
    let custom_error = JsonRpcError::custom(
        -1000,
        "Custom error message".to_string(),
        Some(json!({
            "details": "Additional error information",
            "code": "CUSTOM_ERROR"
        }))
    );
    
    assert_eq!(custom_error.code, -1000);
    assert_eq!(custom_error.message, "Custom error message");
    assert!(custom_error.data.is_some());
}

#[test]
fn test_scan_project_params_deserialization() {
    let json_str = r#"{
        "workspace_path": "/path/to/workspace",
        "file_patterns": ["**/*.rs", "**/*.ts", "**/*.js"]
    }"#;
    
    let params: ScanProjectParams = serde_json::from_str(json_str).unwrap();
    
    assert_eq!(params.workspace_path, "/path/to/workspace");
    assert_eq!(params.file_patterns, Some(vec![
        "**/*.rs".to_string(),
        "**/*.ts".to_string(),
        "**/*.js".to_string()
    ]));
}

#[test]
fn test_scan_project_params_optional_patterns() {
    let json_str = r#"{
        "workspace_path": "/path/to/workspace"
    }"#;
    
    let params: ScanProjectParams = serde_json::from_str(json_str).unwrap();
    
    assert_eq!(params.workspace_path, "/path/to/workspace");
    assert_eq!(params.file_patterns, None);
}

#[test]
fn test_scan_project_result_serialization() {
    let result = ScanProjectResult {
        files_scanned: 42,
        tasks_found: 15,
        errors: vec![
            "Error in file1.rs".to_string(),
            "Error in file2.rs".to_string(),
        ],
    };
    
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: ScanProjectResult = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.files_scanned, 42);
    assert_eq!(deserialized.tasks_found, 15);
    assert_eq!(deserialized.errors.len(), 2);
}

#[test]
fn test_get_tasks_params_deserialization() {
    let json_str = r#"{
        "section": "dev",
        "status": "todo"
    }"#;
    
    let params: GetTasksParams = serde_json::from_str(json_str).unwrap();
    
    assert_eq!(params.section, Some("dev".to_string()));
    assert_eq!(params.status, Some("todo".to_string()));
}

#[test]
fn test_update_task_status_params() {
    let params = UpdateTaskStatusParams {
        section: "dev".to_string(),
        task_id: "task_1".to_string(),
        status: "in_progress".to_string(),
    };
    
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: UpdateTaskStatusParams = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.section, "dev");
    assert_eq!(deserialized.task_id, "task_1");
    assert_eq!(deserialized.status, "in_progress");
}

#[test]
fn test_delete_task_params() {
    let params = DeleteTaskParams {
        section: "dev".to_string(),
        task_id: "task_to_delete".to_string(),
    };
    
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: DeleteTaskParams = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.section, "dev");
    assert_eq!(deserialized.task_id, "task_to_delete");
}

#[test]
fn test_create_task_params() {
    let params = CreateTaskParams {
        section: "dev".to_string(),
        task_id: "new_task".to_string(),
        title: "New Task Title".to_string(),
        description: Some("Detailed description".to_string()),
    };
    
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: CreateTaskParams = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.section, "dev");
    assert_eq!(deserialized.task_id, "new_task");
    assert_eq!(deserialized.title, "New Task Title");
    assert_eq!(deserialized.description, Some("Detailed description".to_string()));
}

#[test]
fn test_find_task_references_params() {
    let params = FindTaskReferencesParams {
        section: "ref".to_string(),
        task_id: "cleanup_task".to_string(),
    };
    
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: FindTaskReferencesParams = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.section, "ref");
    assert_eq!(deserialized.task_id, "cleanup_task");
}

#[test]
fn test_task_reference_serialization() {
    let reference = TaskReference {
        file_path: "src/main.rs".to_string(),
        line: 42,
        note: Some("Important implementation".to_string()),
    };
    
    let serialized = serde_json::to_string(&reference).unwrap();
    let deserialized: TaskReference = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.file_path, "src/main.rs");
    assert_eq!(deserialized.line, 42);
    assert_eq!(deserialized.note, Some("Important implementation".to_string()));
}

#[test]
fn test_jsonrpc_server_success_response() {
    let result = json!({
        "success": true,
        "message": "Operation completed"
    });
    
    let response = JsonRpcServer::success_response(
        Some(Value::Number(serde_json::Number::from(1))),
        result.clone()
    );
    
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.result, Some(result));
    assert!(response.error.is_none());
    assert_eq!(response.id, Some(Value::Number(serde_json::Number::from(1))));
}

#[test]
fn test_jsonrpc_server_error_response() {
    let error = JsonRpcError::custom(
        -1,
        "Test error".to_string(),
        None
    );
    
    let response = JsonRpcServer::error_response(
        Some(Value::Number(serde_json::Number::from(1))),
        error.clone()
    );
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, error.code);
    assert_eq!(response.id, Some(Value::Number(serde_json::Number::from(1))));
}

#[test]
fn test_jsonrpc_request_without_params() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_tasks".to_string(),
        params: None,
        id: Some(Value::Number(serde_json::Number::from(1))),
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(!serialized.contains("\"params\""));
    
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.params.is_none());
}

#[test]
fn test_jsonrpc_notification_request() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "notification".to_string(),
        params: Some(json!({"message": "test"})),
        id: None, // Notification has no ID
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
    
    assert!(deserialized.id.is_none());
    assert_eq!(deserialized.method, "notification");
}

#[test]
fn test_complex_json_structures() {
    let complex_params = json!({
        "workspace_path": "/complex/path",
        "options": {
            "recursive": true,
            "ignore_patterns": ["*.tmp", "*.log"],
            "max_depth": 5
        },
        "filters": [
            {
                "type": "extension",
                "values": ["rs", "ts", "js"]
            },
            {
                "type": "size",
                "min": 0,
                "max": 1048576
            }
        ]
    });
    
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "advanced_scan".to_string(),
        params: Some(complex_params.clone()),
        id: Some(Value::Number(serde_json::Number::from(999))),
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.params, Some(complex_params));
}

#[test]
fn test_error_response_with_data() {
    let error_data = json!({
        "file": "src/parser.rs",
        "line": 42,
        "column": 15,
        "suggestion": "Check syntax"
    });
    
    let error = JsonRpcError::custom(
        -1001,
        "Parse error in file".to_string(),
        Some(error_data.clone())
    );
    
    let response = JsonRpcServer::error_response(
        Some(Value::String("req_123".to_string())),
        error
    );
    
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.data, Some(error_data));
    assert_eq!(response.id, Some(Value::String("req_123".to_string())));
}

#[tokio::test]
async fn test_jsonrpc_client_creation() {
    let (client, _response_tx, _request_rx) = JsonRpcClient::new();
    
    // Тест отправки запроса
    let result = client.send_request(
        "test_method".to_string(),
        Some(json!({"test": "param"}))
    ).await;
    
    assert!(result.is_ok());
}

#[test]
fn test_invalid_json_handling() {
    // Тест парсинга невалидного JSON
    let invalid_json = r#"{"jsonrpc": "2.0", "method": "test", invalid}"#;
    let result: Result<JsonRpcRequest, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_missing_required_fields() {
    // Тест отсутствующих обязательных полей
    let incomplete_json = r#"{"jsonrpc": "2.0"}"#;
    let result: Result<JsonRpcRequest, _> = serde_json::from_str(incomplete_json);
    assert!(result.is_err());
}