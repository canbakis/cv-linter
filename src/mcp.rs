use cv_linter::{
    ExtractError, extract_document, lint_document_with_allow_words, read_selected_file,
};
use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct FileRequest {
    /// Absolute path to one PDF, DOCX, or Markdown CV selected for this operation.
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct LintRequest {
    /// Absolute path to one PDF, DOCX, or Markdown CV selected for this operation.
    path: String,
    /// Up to 256 spelling terms accepted for this operation only. Each item must be one
    /// token of at most 360 bytes; values are not persisted.
    #[serde(default)]
    #[schemars(length(max = 256), inner(length(min = 1, max = 360)))]
    allow_words: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CvLinterMcp;

impl CvLinterMcp {
    pub fn new() -> Self {
        Self
    }
}

#[tool_router(server_handler)]
impl CvLinterMcp {
    #[tool(
        description = "Run deterministic local checks on one explicitly selected PDF, DOCX, or Markdown CV. Non-spelling and spelling findings are returned separately. The checks provide source-derived evidence for parsing and ATS-oriented risk review, but do not establish compatibility with any ATS. This tool does not call a model or use the network."
    )]
    async fn lint_cv(&self, Parameters(request): Parameters<LintRequest>) -> CallToolResult {
        let runtime = tokio::runtime::Handle::current();
        worker_result(
            tokio::task::spawn_blocking(move || {
                let (path, input) = read_request(&request.path)?;
                runtime
                    .block_on(lint_document_with_allow_words(
                        &path,
                        &input,
                        &request.allow_words,
                    ))
                    .map_err(extraction_failure)
            })
            .await,
        )
    }

    #[tool(
        description = "Extract ordered, source-located blocks from one explicitly selected PDF, DOCX, or Markdown CV for host-side ATS-oriented and CV best-practice review. The returned CV text enters the MCP host context and follows that host/model's data policies."
    )]
    async fn extract_cv_text(
        &self,
        Parameters(request): Parameters<FileRequest>,
    ) -> CallToolResult {
        let runtime = tokio::runtime::Handle::current();
        worker_result(
            tokio::task::spawn_blocking(move || {
                let (path, input) = read_request(&request.path)?;
                runtime
                    .block_on(extract_document(&path, &input))
                    .map_err(extraction_failure)
            })
            .await,
        )
    }
}

fn read_request(path: &str) -> Result<(PathBuf, Vec<u8>), ToolFailure> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(ToolFailure::new(
            "invalid_path",
            "the selected document path must be absolute".to_owned(),
        ));
    }

    let input = read_selected_file(path)
        .map_err(|error| ToolFailure::new("input_read_failed", error.to_string()))?;
    Ok((path.to_owned(), input))
}

fn extraction_failure(error: ExtractError) -> ToolFailure {
    let code = match &error {
        ExtractError::UnsupportedFormat { .. } => "unsupported_format",
        ExtractError::TooManyAllowWords { .. } | ExtractError::InvalidAllowWord { .. } => {
            "invalid_options"
        }
        _ => "invalid_document",
    };
    ToolFailure::new(code, error.to_string())
}

fn worker_result<T: serde::Serialize>(
    result: Result<Result<T, ToolFailure>, tokio::task::JoinError>,
) -> CallToolResult {
    match result {
        Ok(Ok(value)) => structured(value),
        Ok(Err(error)) => tool_error(error.code, error.message),
        Err(_) => tool_error(
            "internal_error",
            "the local analysis worker failed".to_owned(),
        ),
    }
}

#[derive(Debug)]
struct ToolFailure {
    code: &'static str,
    message: String,
}

impl ToolFailure {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
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
