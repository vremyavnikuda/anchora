/*!
 * Unified Error Handling Macros for Anchora Backend
 * 
 * This module provides macros for consistent error handling across the Rust backend
 * with automatic integration to the VSCode extension's debug system.
 */

use crate::communication::JsonRpcError;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Enhanced error information that includes debug context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function: String,
    pub method_name: Option<String>,
    pub request_id: Option<Value>,
    pub additional_data: HashMap<String, Value>,
}

impl ErrorContext {
    pub fn new(operation: &str, file: &str, line: u32, column: u32, function: &str) -> Self {
        Self {
            operation: operation.to_string(),
            file: file.to_string(),
            line,
            column,
            function: function.to_string(),
            method_name: None,
            request_id: None,
            additional_data: HashMap::new(),
        }
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method_name = Some(method.to_string());
        self
    }

    pub fn with_request_id(mut self, id: Option<Value>) -> Self {
        self.request_id = id;
        self
    }

    pub fn with_data<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.additional_data.insert(key.into(), value.into());
        self
    }
}

/// Convert anyhow::Error to JsonRpcError with rich context
pub fn create_enhanced_error(
    error: &anyhow::Error,
    context: &ErrorContext,
    error_code: i32,
) -> JsonRpcError {
    let error_message = format!("{}: {}", context.operation, error);
    let debug_data = json!({
        "operation": context.operation,
        "error_source": error.to_string(),
        "error_chain": format!("{:?}", error),
        "location": {
            "file": context.file,
            "line": context.line,
            "column": context.column,
            "function": context.function
        },
        "method": context.method_name,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "additional_data": context.additional_data
    });
    JsonRpcError::custom(error_code, error_message, Some(debug_data))
}

/// Main macro for handling JSON-RPC method calls with unified error handling
#[macro_export]
macro_rules! handle_jsonrpc_method {
    (
        $request_id:expr,
        $method_name:expr,
        $operation:expr,
        $result:expr
    ) => {{
        let context = $crate::error_macros::ErrorContext::new(
            $operation,
            file!(),
            line!(),
            column!(),
            module_path!(),
        )
        .with_method($method_name)
        .with_request_id($request_id.clone());
        match $result {
            Ok(value) => {
                eprintln!("[DEBUG] Operation '{}' completed successfully", $operation);
                let json_value = match serde_json::to_value(&value) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[ERROR] Failed to serialize result for {}: {}", $operation, e);
                        serde_json::Value::Null
                    }
                };
                $crate::communication::JsonRpcServer::success_response($request_id, json_value)
            }
            Err(error) => {
                let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -1);
                eprintln!("[ERROR] Operation '{}' failed: {}", $operation, error);
                eprintln!("[ERROR] Context: {}:{} in {}", file!(), line!(), module_path!());
                $crate::communication::JsonRpcServer::error_response($request_id, enhanced_error)
            }
        }
    }};
}

/// Macro for handling parameter parsing with automatic error response
#[macro_export]
macro_rules! parse_params {
    ($params:expr, $param_type:ty, $request_id:expr, $method_name:expr) => {{
        match $params {
            Some(params) => {
                match serde_json::from_value::<$param_type>(params) {
                    Ok(parsed_params) => Ok(parsed_params),
                    Err(e) => {
                        let context = $crate::error_macros::ErrorContext::new(
                            &format!("Parse {} parameters", stringify!($param_type)),
                            file!(),
                            line!(),
                            column!(),
                            module_path!(),
                        )
                        .with_method($method_name)
                        .with_request_id($request_id.clone());
                        let error = anyhow::anyhow!("Parameter parsing failed: {}", e);
                        let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -32602);
                        eprintln!("[ERROR] Parameter parsing failed for {}: {}", $method_name, e);
                        return $crate::communication::JsonRpcServer::error_response($request_id, enhanced_error);
                    }
                }
            }
            None => {
                eprintln!("[ERROR] Missing required parameters for method: {}", $method_name);
                return $crate::communication::JsonRpcServer::error_response(
                    $request_id,
                    $crate::communication::JsonRpcError::invalid_params()
                );
            }
        }
    }};
}

/// Simplified macro for methods that don't require parameters
#[macro_export]
macro_rules! handle_simple_method {
    (
        $request_id:expr,
        $method_name:expr,
        $operation:expr,
        $async_call:expr
    ) => {{
        let result = $async_call.await;
        handle_jsonrpc_method!($request_id, $method_name, $operation, result)
    }};
}

/// Macro for methods with required parameters
#[macro_export]
macro_rules! handle_parameterized_method {
    (
        $request:expr,
        $param_type:ty,
        $method_name:expr,
        $operation:expr,
        |$params:ident| $async_call:expr
    ) => {{
        match $request.params {
            Some(params) => {
                match serde_json::from_value::<$param_type>(params) {
                    Ok($params) => {
                        let result = $async_call.await;
                        handle_jsonrpc_method!($request.id, $method_name, $operation, result)
                    }
                    Err(e) => {
                        let context = $crate::error_macros::ErrorContext::new(
                            &format!("Parse {} parameters", stringify!($param_type)),
                            file!(),
                            line!(),
                            column!(),
                            module_path!(),
                        )
                        .with_method($method_name)
                        .with_request_id($request.id.clone());
                        let error = anyhow::anyhow!("Parameter parsing failed: {}", e);
                        let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -32602);
                        eprintln!("[ERROR] Parameter parsing failed for {}: {}", $method_name, e);
                        $crate::communication::JsonRpcServer::error_response($request.id, enhanced_error)
                    }
                }
            }
            None => {
                eprintln!("[ERROR] Missing required parameters for method: {}", $method_name);
                $crate::communication::JsonRpcServer::error_response(
                    $request.id,
                    $crate::communication::JsonRpcError::invalid_params()
                )
            }
        }
    }};
}

/// Enhanced error logging that can be integrated with VSCode extension debug system
pub fn log_error_to_debug_channel(
    operation: &str,
    error: &anyhow::Error,
    context: &ErrorContext,
) {
    let structured_log = json!({
        "level": "ERROR",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "operation": operation,
        "error": error.to_string(),
        "context": {
            "file": context.file,
            "line": context.line,
            "function": context.function,
            "method": context.method_name
        },
        "debug_data": context.additional_data
    });
    eprintln!("ANCHORA_DEBUG: {}", structured_log);
}

/// Macro to add debug context to any operation
#[macro_export]
macro_rules! debug_context {
    ($operation:expr, $($key:expr => $value:expr),*) => {{
        let mut context = $crate::error_macros::ErrorContext::new(
            $operation,
            file!(),
            line!(),
            column!(),
            module_path!(),
        );
        $(
            context = context.with_data($key, $value);
        )*
        context
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation", "test.rs", 42, 10, "test_module")
            .with_method("test_method")
            .with_data("param1", "value1");
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.file, "test.rs");
        assert_eq!(context.line, 42);
        assert_eq!(context.function, "test_module");
        assert_eq!(context.method_name, Some("test_method".to_string()));
        assert_eq!(context.additional_data.get("param1"), Some(&json!("value1")));
    }

    #[test]
    fn test_enhanced_error_creation() {
        let error = anyhow::anyhow!("Test error");
        let context = ErrorContext::new("test_operation", "test.rs", 42, 10, "test_module");
        let json_error = create_enhanced_error(&error, &context, -1000);
        assert_eq!(json_error.code, -1000);
        assert!(json_error.message.contains("test_operation"));
        assert!(json_error.data.is_some());
        if let Some(data) = json_error.data {
            assert!(data.get("location").is_some());
            assert!(data.get("timestamp").is_some());
        }
    }

    fn mock_successful_operation() -> Result<serde_json::Value> {
        Ok(json!({"success": true}))
    }

    fn mock_failing_operation() -> Result<serde_json::Value> {
        Err(anyhow::anyhow!("Mock error"))
    }

    #[test]
    fn test_macro_success_case() {
        let response = handle_jsonrpc_method!(
            Some(json!(1)),
            "test_method",
            "test_operation",
            mock_successful_operation()
        );

        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_macro_error_case() {
        let response = handle_jsonrpc_method!(
            Some(json!(1)),
            "test_method", 
            "test_operation",
            mock_failing_operation()
        );
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        if let Some(error) = response.error {
            assert_eq!(error.code, -1);
            assert!(error.message.contains("test_operation"));
            assert!(error.data.is_some());
        }
    }
}