use cv_linter::{extract_plain_text, lint_plain_text, read_selected_file};
use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct FileRequest {
    /// Path to one explicitly selected UTF-8 plain-text CV.
    path: String,
}

#[derive(Debug, Clone)]
pub struct CvLinterMcp;

#[tool_router(server_handler)]
impl CvLinterMcp {
    #[tool(
        description = "Run deterministic local checks on one explicitly selected UTF-8 plain-text CV. This tool does not call a model or use the network."
    )]
    fn lint_cv(&self, Parameters(request): Parameters<FileRequest>) -> CallToolResult {
        let input = match read_selected_file(Path::new(&request.path)) {
            Ok(input) => input,
            Err(error) => return tool_error("input_read_failed", error.to_string()),
        };

        match lint_plain_text(&input) {
            Ok(report) => structured(report),
            Err(error) => tool_error("invalid_text", error.to_string()),
        }
    }

    #[tool(
        description = "Extract ordered, source-located blocks from one explicitly selected UTF-8 plain-text CV. The returned CV text enters the MCP host context and follows that host/model's data policies."
    )]
    fn extract_cv_text(&self, Parameters(request): Parameters<FileRequest>) -> CallToolResult {
        let input = match read_selected_file(Path::new(&request.path)) {
            Ok(input) => input,
            Err(error) => return tool_error("input_read_failed", error.to_string()),
        };

        match extract_plain_text(&input) {
            Ok(document) => structured(document),
            Err(error) => tool_error("invalid_text", error.to_string()),
        }
    }
}

fn structured(value: impl serde::Serialize) -> CallToolResult {
    let value = serde_json::to_value(value).expect("CV Linter reports must serialize");
    CallToolResult::structured(value)
}

fn tool_error(code: &'static str, message: String) -> CallToolResult {
    CallToolResult::structured_error(json!({
        "error": {
            "code": code,
            "message": message,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn tool_error_is_machine_readable() {
        let result = tool_error("invalid_text", "bad input".to_owned());
        let structured = result.structured_content.unwrap_or(Value::Null);

        assert_eq!(result.is_error, Some(true));
        assert_eq!(structured["error"]["code"], "invalid_text");
    }
}
