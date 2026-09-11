mod common;

use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tempfile::tempdir;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn cli_help_succeeds_and_usage_errors_use_the_operational_exit_code() {
    let help = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));

    let invalid = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .arg("lint")
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("--input"));
}

#[test]
fn cli_spelling_allow_words_are_operation_scoped() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.md");
    fs::write(&input_path, "# Profil\n\nquuxzorp\n").unwrap();

    let default_output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(default_output.status.success());
    let default_report: Value = serde_json::from_slice(&default_output.stdout).unwrap();
    assert_eq!(default_report["schema_version"], "0.3.0");
    assert!(
        default_report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| !finding["rule_id"]
                .as_str()
                .unwrap()
                .starts_with("cv.spelling."))
    );
    assert!(
        default_report["spelling_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule_id"] == "cv.spelling.unknown_word"
                && finding["message"].as_str().unwrap().contains("quuxzorp"))
    );

    let allowed_output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(&input_path)
        .args(["--allow-word", "quuxzorp"])
        .output()
        .unwrap();
    assert!(allowed_output.status.success());
    let allowed_report: Value = serde_json::from_slice(&allowed_output.stdout).unwrap();
    assert!(
        allowed_report["spelling_findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["rule_id"] != "cv.spelling.unknown_word")
    );
}

#[test]
fn cli_extracts_all_mvp_formats() {
    let directory = tempdir().unwrap();
    let fixtures = [
        (
            "cv.md",
            include_bytes!("fixtures/swedish-cv.md").to_vec(),
            "Göteborg",
        ),
        (
            "cv.pdf",
            common::text_pdf(&[
                "Alex Andersson",
                "alex.andersson@example.com",
                "Senior Rustutvecklare",
            ]),
            "Rustutvecklare",
        ),
        (
            "cv.docx",
            common::docx(&[
                "Alex Andersson",
                "alex.andersson@example.com",
                "Rustutvecklare i Göteborg",
            ]),
            "Göteborg",
        ),
    ];

    for (name, input, expected_text) in fixtures {
        let input_path = directory.path().join(name);
        fs::write(&input_path, input).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
            .args(["extract-text", "--input"])
            .arg(&input_path)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let document: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(document["status"], "ok");
        assert_eq!(document["extractor"]["name"], "xberg");
        assert!(document["blocks"].as_array().unwrap().iter().any(|block| {
            block["text"]
                .as_str()
                .is_some_and(|text| text.contains(expected_text))
        }));
    }
}

#[test]
fn extract_text_cli_emits_json_without_diagnostics_on_stdout() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "Profil\nUtvecklare\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["extract-text", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["blocks"][0]["id"], "block-0001");
}

#[test]
fn lint_cli_uses_distinct_finding_and_operational_exit_codes() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "Role\0\n").unwrap();

    let finding_output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();
    assert_eq!(finding_output.status.code(), Some(1));
    assert!(finding_output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&finding_output.stdout).unwrap();
    assert_eq!(report["findings"][0]["severity"], "error");

    fs::write(&input_path, "Role \n").unwrap();
    let informational_output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(informational_output.status.success());
    let report: Value = serde_json::from_slice(&informational_output.stdout).unwrap();
    assert_eq!(report["findings"][0]["severity"], "info");

    let operational_output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(directory.path())
        .output()
        .unwrap();
    assert_eq!(operational_output.status.code(), Some(3));
    assert!(operational_output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&operational_output.stderr).contains("not a regular file"));
}

#[test]
fn closed_stdout_pipe_does_not_panic() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "entry\n".repeat(5_000)).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["extract-text", "--input"])
        .arg(&input_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut prefix = [0; 64];
    stdout.read_exact(&mut prefix).unwrap();
    drop(stdout);

    let status = wait_for_child(&mut child);
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();

    assert!(status.success());
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn stdio_mcp_lists_and_calls_both_tools() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "Profil\nUtvecklare\0\n").unwrap();
    let mut mcp = McpClient::start();

    let tools = mcp.request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }));
    let names = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["extract_cv_text", "lint_cv"]);
    let lint_schema = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "lint_cv")
        .unwrap()["inputSchema"]
        .clone();
    assert_eq!(lint_schema["properties"]["allow_words"]["maxItems"], 256);
    assert_eq!(
        lint_schema["properties"]["allow_words"]["items"]["maxLength"],
        360
    );
    assert!(
        lint_schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .all(|field| field != "allow_words")
    );

    let extraction = mcp.call_tool(3, "extract_cv_text", &input_path);
    assert_eq!(extraction["result"]["isError"], false);
    assert_eq!(
        extraction["result"]["structuredContent"]["blocks"][0]["id"],
        "block-0001"
    );

    let lint = mcp.call_tool(4, "lint_cv", &input_path);
    assert_eq!(lint["result"]["isError"], false);
    assert_eq!(
        lint["result"]["structuredContent"]["findings"][0]["rule_id"],
        "text.nul_character"
    );

    mcp.shutdown();
}

#[test]
fn stdio_mcp_and_cli_return_equivalent_markdown_results() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.md");
    fs::write(&input_path, include_bytes!("fixtures/swedish-cv.md")).unwrap();

    let cli_extract = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["extract-text", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(cli_extract.status.success());
    let cli_extract: Value = serde_json::from_slice(&cli_extract.stdout).unwrap();

    let cli_lint = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(cli_lint.status.success());
    let cli_lint: Value = serde_json::from_slice(&cli_lint.stdout).unwrap();

    let mut mcp = McpClient::start();
    let mcp_extract = mcp.call_tool(2, "extract_cv_text", &input_path);
    let mcp_lint = mcp.call_tool(3, "lint_cv", &input_path);

    assert_eq!(mcp_extract["result"]["structuredContent"], cli_extract);
    assert_eq!(mcp_lint["result"]["structuredContent"], cli_lint);
    mcp.shutdown();
}

#[test]
fn stdio_mcp_accepts_bounded_operation_scoped_spelling_allow_words() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.md");
    fs::write(&input_path, "# Profil\n\nquuxzorp\n").unwrap();
    let mut mcp = McpClient::start();

    let before = mcp.call_tool(2, "lint_cv", &input_path);
    assert!(
        before["result"]["structuredContent"]["spelling_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule_id"] == "cv.spelling.unknown_word")
    );

    let allowed = mcp.call_tool_with_arguments(
        3,
        "lint_cv",
        json!({ "path": &input_path, "allow_words": ["quuxzorp"] }),
    );
    assert_eq!(allowed["result"]["isError"], false);
    assert!(
        allowed["result"]["structuredContent"]["spelling_findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["rule_id"] != "cv.spelling.unknown_word")
    );
    assert_eq!(
        allowed["result"]["structuredContent"]["options"]["spelling_allow_words"],
        json!(["quuxzorp"])
    );

    let after = mcp.call_tool(4, "lint_cv", &input_path);
    assert!(
        after["result"]["structuredContent"]["spelling_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule_id"] == "cv.spelling.unknown_word")
    );

    let invalid = mcp.call_tool_with_arguments(
        5,
        "lint_cv",
        json!({ "path": &input_path, "allow_words": ["two words"] }),
    );
    assert_eq!(invalid["result"]["isError"], true);
    assert_eq!(
        invalid["result"]["structuredContent"]["error"]["code"],
        "invalid_options"
    );
    mcp.shutdown();
}

#[test]
fn stdio_mcp_accepts_any_selected_absolute_path() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("selected.md");
    fs::write(&input_path, "# Profil\n\nUtvecklare\n").unwrap();
    let mut mcp = McpClient::start();

    let extraction = mcp.call_tool(2, "extract_cv_text", &input_path);
    assert_eq!(extraction["result"]["isError"], false);
    assert_eq!(
        extraction["result"]["structuredContent"]["blocks"][0]["id"],
        "block-0001"
    );

    mcp.shutdown();
}

struct McpClient {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}

impl McpClient {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
            .args(["mcp", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, responses) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                match stdout.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let response = serde_json::from_str(&line)
                            .map_err(|error| format!("invalid MCP response: {error}"));
                        if sender.send(response).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(format!("failed to read MCP response: {error}")));
                        break;
                    }
                }
            }
        });

        let mut client = Self {
            child,
            stdin: Some(stdin),
            responses,
            reader: Some(reader),
        };
        let initialize = client.request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "cv-linter-test", "version": "0.1.0" }
            }
        }));
        assert_eq!(initialize["id"], 1);
        client.notify(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }));
        client
    }

    fn call_tool(&mut self, id: u64, name: &str, path: &std::path::Path) -> Value {
        self.call_tool_with_arguments(id, name, json!({ "path": path }))
    }

    fn call_tool_with_arguments(&mut self, id: u64, name: &str, arguments: Value) -> Value {
        self.request(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        }))
    }

    fn request(&mut self, message: Value) -> Value {
        self.notify(message);
        self.responses
            .recv_timeout(PROCESS_TIMEOUT)
            .expect("timed out waiting for MCP response")
            .unwrap()
    }

    fn notify(&mut self, message: Value) {
        let stdin = self.stdin.as_mut().expect("MCP stdin is closed");
        writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
        stdin.flush().unwrap();
    }

    fn shutdown(&mut self) {
        drop(self.stdin.take());
        let status = wait_for_child(&mut self.child);
        let mut stderr = String::new();
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        if let Some(reader) = self.reader.take() {
            reader.join().unwrap();
        }

        assert!(status.success());
        assert!(stderr.is_empty(), "{stderr}");
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn wait_for_child(child: &mut Child) -> ExitStatus {
    let deadline = Instant::now() + PROCESS_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("timed out waiting for child process");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
