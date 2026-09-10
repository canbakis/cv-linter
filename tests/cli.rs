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
fn mcp_requires_an_explicit_allowed_root() {
    let output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["mcp", "--stdio"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--allow-root"));
}

#[test]
fn stdio_mcp_lists_and_calls_both_tools() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "Profil\nUtvecklare\0\n").unwrap();
    let mut mcp = McpClient::start(directory.path());

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
fn stdio_mcp_rejects_paths_outside_the_allowed_root() {
    let allowed = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let outside_path = outside.path().join("private.txt");
    fs::write(&outside_path, "not authorized\n").unwrap();
    let mut mcp = McpClient::start(allowed.path());

    assert_path_not_allowed(mcp.call_tool(2, "extract_cv_text", &outside_path));
    assert_path_not_allowed(mcp.call_tool(
        3,
        "extract_cv_text",
        std::path::Path::new("relative.txt"),
    ));

    let parent_path = allowed.path().join("nested").join("..").join("cv.txt");
    assert_path_not_allowed(mcp.call_tool(4, "extract_cv_text", &parent_path));

    #[cfg(unix)]
    {
        let link_path = allowed.path().join("linked.txt");
        std::os::unix::fs::symlink(&outside_path, &link_path).unwrap();
        assert_path_not_allowed(mcp.call_tool(5, "extract_cv_text", &link_path));
    }

    mcp.shutdown();
}

fn assert_path_not_allowed(response: Value) {
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        "path_not_allowed"
    );
}

struct McpClient {
    child: Child,
    stdin: Option<ChildStdin>,
    responses: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}

impl McpClient {
    fn start(allowed_root: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
            .args(["mcp", "--stdio", "--allow-root"])
            .arg(allowed_root)
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
        self.request(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": { "path": path }
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
