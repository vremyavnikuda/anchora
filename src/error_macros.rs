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
/// Enhanced for server-side logic migration with performance metrics
#[macro_export]
macro_rules! handle_jsonrpc_method {
    (
        $request_id:expr,
        $method_name:expr,
        $operation:expr,
        $result:expr
    ) => {{
        let start_time = std::time::Instant::now();
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
                let duration = start_time.elapsed();
                eprintln!("[DEBUG] Operation '{}' completed successfully in {:?}", $operation, duration);
                let json_value = match serde_json::to_value(&value) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[ERROR] Failed to serialize result for {}: {}", $operation, e);
                        serde_json::Value::Null
                    }
                };
                if $method_name.starts_with("search_") || $method_name.starts_with("get_statistics") || $method_name.starts_with("validate_") {
                    if let serde_json::Value::Object(mut obj) = json_value {
                        obj.insert("_performance".to_string(), serde_json::json!({
                            "duration_ms": duration.as_millis(),
                            "operation": $operation,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }));
                        $crate::communication::JsonRpcServer::success_response($request_id, serde_json::Value::Object(obj))
                    } else {
                        $crate::communication::JsonRpcServer::success_response($request_id, json_value)
                    }
                } else {
                    $crate::communication::JsonRpcServer::success_response($request_id, json_value) 
                }
            }
            Err(error) => {
                let duration = start_time.elapsed();
                let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -1);
                eprintln!("[ERROR] Operation '{}' failed after {:?}: {}", $operation, duration, error);
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
/// Extended for server-side operations monitoring
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

/// Log performance metrics for server-side operations
pub fn log_performance_metrics(
    operation: &str,
    duration: std::time::Duration,
    additional_metrics: Option<serde_json::Value>,
) {
    let metrics = json!({
        "level": "PERFORMANCE",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "operation": operation,
        "duration_ms": duration.as_millis(),
        "duration_micros": duration.as_micros(),
        "additional_metrics": additional_metrics.unwrap_or(serde_json::Value::Null)
    });
    eprintln!("ANCHORA_PERF: {}", metrics);
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

/// Macro for handling server-side search operations with performance tracking
#[macro_export]
macro_rules! handle_search_operation {
    (
        $request:expr,
        $param_type:ty,
        $operation:expr,
        |$params:ident| $search_call:expr
    ) => {{
        let start_time = std::time::Instant::now();
        match $request.params {
            Some(params) => {
                match serde_json::from_value::<$param_type>(params) {
                    Ok($params) => {
                        let search_start = std::time::Instant::now();
                        let result = $search_call;
                        let search_duration = search_start.elapsed();
                        
                        match result {
                            Ok(mut search_result) => {
                                if let Ok(mut json_result) = serde_json::to_value(&search_result) {
                                    if let serde_json::Value::Object(ref mut obj) = json_result {
                                        obj.insert("performance_metrics".to_string(), serde_json::json!({
                                            "search_duration_ms": search_duration.as_millis(),
                                            "total_duration_ms": start_time.elapsed().as_millis(),
                                            "operation": $operation,
                                            "timestamp": chrono::Utc::now().to_rfc3339()
                                        }));
                                    }
                                    $crate::error_macros::log_performance_metrics(
                                        $operation,
                                        search_duration,
                                        Some(serde_json::json!({"search_type": "indexed"}))
                                    );
                                    $crate::communication::JsonRpcServer::success_response($request.id, json_result)
                                } else {
                                    $crate::communication::JsonRpcServer::success_response($request.id, serde_json::to_value(&search_result).unwrap_or(serde_json::Value::Null))
                                }
                            }
                            Err(error) => {
                                let context = $crate::error_macros::ErrorContext::new(
                                    $operation,
                                    file!(),
                                    line!(),
                                    column!(),
                                    module_path!()
                                ).with_request_id($request.id.clone())
                                 .with_data("search_duration_ms", search_duration.as_millis());
                                let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -1);
                                eprintln!("[ERROR] Search operation '{}' failed after {:?}: {}", $operation, search_duration, error);
                                $crate::communication::JsonRpcServer::error_response($request.id, enhanced_error)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Parameter parsing failed for search operation: {}", e);
                        $crate::communication::JsonRpcServer::error_response(
                            $request.id,
                            $crate::communication::JsonRpcError::invalid_params()
                        )
                    }
                }
            }
            None => {
                eprintln!("[ERROR] Missing required parameters for search operation");
                $crate::communication::JsonRpcServer::error_response(
                    $request.id,
                    $crate::communication::JsonRpcError::invalid_params()
                )
            }
        }
    }};
}

/// Macro for handling server-side statistics operations with caching support
#[macro_export]
macro_rules! handle_statistics_operation {
    (
        $request:expr,
        $operation:expr,
        $stats_call:expr
    ) => {{
        let start_time = std::time::Instant::now();
        let cache_start = std::time::Instant::now();
        let result = $stats_call;
        let cache_duration = cache_start.elapsed();
        
        match result {
            Ok(stats_result) => {
                match serde_json::to_value(&stats_result) {
                    Ok(mut json_result) => {
                        if let serde_json::Value::Object(ref mut obj) = json_result {
                            obj.insert("cache_metrics".to_string(), serde_json::json!({
                                "cache_duration_ms": cache_duration.as_millis(),
                                "total_duration_ms": start_time.elapsed().as_millis(),
                                "operation": $operation,
                                "cache_hit": cache_duration.as_millis() < 5, // Assume cache hit if < 5ms
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }));
                        }
                        $crate::error_macros::log_performance_metrics(
                            $operation,
                            cache_duration,
                            Some(serde_json::json!({"operation_type": "statistics", "cached": cache_duration.as_millis() < 5}))
                        );
                        $crate::communication::JsonRpcServer::success_response($request.id, json_result)
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to serialize statistics result: {}", e);
                        $crate::communication::JsonRpcServer::error_response(
                            $request.id,
                            $crate::communication::JsonRpcError::internal_error()
                        )
                    }
                }
            }
            Err(error) => {
                let context = $crate::error_macros::ErrorContext::new(
                    $operation,
                    file!(),
                    line!(),
                    column!(),
                    module_path!()
                ).with_request_id($request.id.clone())
                 .with_data("cache_duration_ms", cache_duration.as_millis());
                let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -1);
                eprintln!("[ERROR] Statistics operation '{}' failed after {:?}: {}", $operation, cache_duration, error);
                $crate::communication::JsonRpcServer::error_response($request.id, enhanced_error)
            }
        }
    }};
}

/// Macro for handling validation operations with context-aware error messages
#[macro_export]
macro_rules! handle_validation_operation {
    (
        $request:expr,
        $param_type:ty,
        $operation:expr,
        |$params:ident| $validation_call:expr
    ) => {{
        match $request.params {
            Some(params) => {
                match serde_json::from_value::<$param_type>(params) {
                    Ok($params) => {
                        let validation_start = std::time::Instant::now();
                        let result = $validation_call;
                        let validation_duration = validation_start.elapsed();
                        
                        match result {
                            Ok(validation_result) => {
                                match serde_json::to_value(&validation_result) {
                                    Ok(mut json_result) => {
                                        if let serde_json::Value::Object(ref mut obj) = json_result {
                                            obj.insert("validation_metrics".to_string(), serde_json::json!({
                                                "validation_duration_ms": validation_duration.as_millis(),
                                                "operation": $operation,
                                                "timestamp": chrono::Utc::now().to_rfc3339()
                                            }));
                                        }
                                        $crate::communication::JsonRpcServer::success_response($request.id, json_result)
                                    }
                                    Err(e) => {
                                        eprintln!("[ERROR] Failed to serialize validation result: {}", e);
                                        $crate::communication::JsonRpcServer::error_response(
                                            $request.id,
                                            $crate::communication::JsonRpcError::internal_error()
                                        )
                                    }
                                }
                            }
                            Err(error) => {
                                let context = $crate::error_macros::ErrorContext::new(
                                    $operation,
                                    file!(),
                                    line!(),
                                    column!(),
                                    module_path!()
                                ).with_request_id($request.id.clone())
                                 .with_data("validation_duration_ms", validation_duration.as_millis());
                                let enhanced_error = $crate::error_macros::create_enhanced_error(&error, &context, -32602);
                                eprintln!("[ERROR] Validation operation '{}' failed: {}", $operation, error);
                                $crate::communication::JsonRpcServer::error_response($request.id, enhanced_error)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Parameter parsing failed for validation operation: {}", e);
                        $crate::communication::JsonRpcServer::error_response(
                            $request.id,
                            $crate::communication::JsonRpcError::invalid_params()
                        )
                    }
                }
            }
            None => {
                eprintln!("[ERROR] Missing required parameters for validation operation");
                $crate::communication::JsonRpcServer::error_response(
                    $request.id,
                    $crate::communication::JsonRpcError::invalid_params()
                )
            }
        }
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