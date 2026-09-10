use cv_linter::{extract_plain_text, lint_plain_text, read_selected_file};
use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct FileRequest {
    /// Absolute path to one UTF-8 plain-text CV beneath a server-configured allowed root.
    path: String,
}

#[derive(Debug, Clone)]
pub struct CvLinterMcp {
    file_access: FileAccessPolicy,
}

impl CvLinterMcp {
    pub fn new(allowed_roots: Vec<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            file_access: FileAccessPolicy::new(allowed_roots)?,
        })
    }
}

#[tool_router(server_handler)]
impl CvLinterMcp {
    #[tool(
        description = "Run deterministic local checks on one UTF-8 plain-text CV beneath a user-configured allowed root. This tool does not call a model or use the network."
    )]
    async fn lint_cv(&self, Parameters(request): Parameters<FileRequest>) -> CallToolResult {
        let file_access = self.file_access.clone();
        worker_result(
            tokio::task::spawn_blocking(move || {
                let input = read_request(&file_access, &request)?;
                lint_plain_text(&input)
                    .map_err(|error| ToolFailure::new("invalid_text", error.to_string()))
            })
            .await,
        )
    }

    #[tool(
        description = "Extract ordered, source-located blocks from one UTF-8 plain-text CV beneath a user-configured allowed root. The returned CV text enters the MCP host context and follows that host/model's data policies."
    )]
    async fn extract_cv_text(
        &self,
        Parameters(request): Parameters<FileRequest>,
    ) -> CallToolResult {
        let file_access = self.file_access.clone();
        worker_result(
            tokio::task::spawn_blocking(move || {
                let input = read_request(&file_access, &request)?;
                extract_plain_text(&input)
                    .map_err(|error| ToolFailure::new("invalid_text", error.to_string()))
            })
            .await,
        )
    }
}

#[derive(Debug, Clone)]
struct FileAccessPolicy {
    allowed_roots: Vec<PathBuf>,
}

impl FileAccessPolicy {
    fn new(allowed_roots: Vec<PathBuf>) -> io::Result<Self> {
        if allowed_roots.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "MCP requires at least one explicit --allow-root directory",
            ));
        }

        let mut canonical_roots = Vec::with_capacity(allowed_roots.len());
        for root in allowed_roots {
            let canonical_root = root.canonicalize()?;
            if !canonical_root.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "an MCP allowed root is not a directory",
                ));
            }
            canonical_roots.push(canonical_root);
        }

        Ok(Self {
            allowed_roots: canonical_roots,
        })
    }

    fn read_selected_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        if !path.is_absolute()
            || path
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err(path_not_allowed());
        }

        let canonical_path = path.canonicalize()?;
        if !self
            .allowed_roots
            .iter()
            .any(|root| canonical_path.starts_with(root))
        {
            return Err(path_not_allowed());
        }

        read_selected_file(&canonical_path)
    }
}

fn path_not_allowed() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "requested path is outside the configured MCP roots",
    )
}

fn read_request(
    file_access: &FileAccessPolicy,
    request: &FileRequest,
) -> Result<Vec<u8>, ToolFailure> {
    file_access
        .read_selected_file(Path::new(&request.path))
        .map_err(|error| {
            let code = if error.kind() == io::ErrorKind::PermissionDenied {
                "path_not_allowed"
            } else {
                "input_read_failed"
            };
            ToolFailure::new(code, error.to_string())
        })
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
