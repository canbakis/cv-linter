use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tempfile::tempdir;

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
fn lint_cli_rejects_a_directory() {
    let directory = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["lint", "--input"])
        .arg(directory.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a regular file"));
}

#[test]
fn stdio_mcp_lists_and_calls_the_two_tools() {
    let directory = tempdir().unwrap();
    let input_path = directory.path().join("cv.txt");
    fs::write(&input_path, "Profil\nUtvecklare\n").unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_cv-linter"))
        .args(["mcp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send_message(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "cv-linter-test", "version": "0.1.0" }
            }
        }),
    );
    let initialize = read_message(&mut stdout);
    assert_eq!(initialize["id"], 1);

    send_message(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );
    send_message(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );
    let tools = read_message(&mut stdout);
    let names = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["extract_cv_text", "lint_cv"]);

    send_message(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "extract_cv_text",
                "arguments": { "path": input_path }
            }
        }),
    );
    let extraction = read_message(&mut stdout);
    assert_eq!(extraction["result"]["isError"], false);
    assert_eq!(
        extraction["result"]["structuredContent"]["blocks"][0]["id"],
        "block-0001"
    );

    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

fn send_message(stdin: &mut impl Write, message: Value) {
    writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
    stdin.flush().unwrap();
}

fn read_message(stdout: &mut impl BufRead) -> Value {
    let mut line = String::new();
    assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "MCP server closed");
    serde_json::from_str(&line).unwrap()
}
