use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub fn invalid_params() -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }
    }

    pub fn custom(code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScanProjectParams {
    pub workspace_path: String,
    pub file_patterns: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanProjectResult {
    pub files_scanned: u32,
    pub tasks_found: u32,
    pub tasks_removed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetTasksParams {
    pub section: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTaskStatusParams {
    pub section: String,
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaskParams {
    pub section: String,
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteTaskParams {
    pub section: String,
    pub task_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FindTaskReferencesParams {
    pub section: String,
    pub task_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskReference {
    pub file_path: String,
    pub line: u32,
    pub note: Option<String>,
}

// Note-related types
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateNoteParams {
    pub title: String,
    pub content: String,
    pub section: String,
    pub suggested_task_id: String,
    pub suggested_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteResponse {
    pub success: bool,
    pub message: String,
    pub note_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateLinkParams {
    pub note_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteNoteParams {
    pub note_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateLinkResponse {
    pub success: bool,
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResponse {
    pub success: bool,
    pub message: String,
}

// New server-side operation parameters
#[derive(Debug, Deserialize)]
pub struct SearchTasksParams {
    pub query: String,
    pub filters: Option<serde_json::Value>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct GetStatisticsParams {
    pub include_trends: Option<bool>,
    pub section_filter: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GetTaskOverviewParams {
    pub include_recent_activity: Option<bool>,
    pub activity_limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateTaskParams {
    pub section: String,
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub check_duplicates: Option<bool>,
    pub suggest_alternatives: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetSuggestionsParams {
    pub partial_query: String,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetFileDecorationsParams {
    pub file_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetFilteredTasksParams {
    pub sections: Option<Vec<String>>,
    pub statuses: Option<Vec<String>>,
    pub created_after: Option<String>,
    pub updated_after: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckConflictsParams {
    pub section: String,
    pub task_id: String,
}

pub trait JsonRpcHandler: Send + Sync {
    fn handle_request(
        &self,
        request: JsonRpcRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = JsonRpcResponse> + Send + '_>>;
}

pub struct JsonRpcServer {
    handler: Box<dyn JsonRpcHandler>,
}

impl JsonRpcServer {
    pub fn new(handler: Box<dyn JsonRpcHandler>) -> Self {
        Self { handler }
    }

    pub async fn run_stdio(&self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        println!("JSON-RPC server started on stdin/stdout");
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("JSON-RPC server shutting down");
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let response = self.process_line(line).await;
                    let response_json = serde_json::to_string(&response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_line(&self, line: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(_) => {
                return JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    result: None,
                    error: Some(JsonRpcError::parse_error()),
                    id: None,
                };
            }
        };
        if request.jsonrpc != JSONRPC_VERSION {
            return JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_request()),
                id: request.id,
            };
        }
        self.handler.handle_request(request).await
    }

    pub fn success_response(id: Option<Value>, result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error_response(id: Option<Value>, error: JsonRpcError) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

pub struct JsonRpcClient {
    tx: mpsc::UnboundedSender<String>,
    rx: mpsc::UnboundedReceiver<String>,
}

impl JsonRpcClient {
    pub fn new() -> (
        Self,
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
    ) {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        let client = Self {
            tx: request_tx,
            rx: response_rx,
        };
        (client, response_tx, request_rx)
    }

    pub async fn send_request(&self, method: String, params: Option<Value>) -> anyhow::Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            params,
            id: Some(Value::Number(serde_json::Number::from(1))),
        };
        let request_json = serde_json::to_string(&request)?;
        self.tx.send(request_json)?;
        Ok(())
    }

    pub async fn receive_response(&mut self) -> Option<JsonRpcResponse> {
        if let Some(response_json) = self.rx.recv().await {
            serde_json::from_str(&response_json).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "scan_project".to_string(),
            params: Some(serde_json::json!({
                "workspace_path": "/path/to/project"
            })),
            id: Some(Value::Number(serde_json::Number::from(1))),
        };
        let json = serde_json::to_string(&request).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "scan_project");
        assert_eq!(parsed.jsonrpc, "2.0");
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "files_scanned": 42,
                "tasks_found": 15
            })),
            error: None,
            id: Some(Value::Number(serde_json::Number::from(1))),
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.result.is_some());
        assert!(parsed.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error() {
        let error = JsonRpcError::method_not_found();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }
}
