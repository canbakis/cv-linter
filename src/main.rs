use clap::{Parser, Subcommand};
mod mcp;

use cv_linter::{Severity, extract_plain_text, lint_plain_text, read_selected_file};
use rmcp::{ServiceExt, transport::stdio};
use serde::Serialize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run deterministic checks on one UTF-8 text CV.
    Lint(InputArgs),
    /// Extract ordered, source-located text blocks from one UTF-8 text CV.
    ExtractText(InputArgs),
    /// Start the local MCP server.
    Mcp {
        /// Serve MCP over standard input and output.
        #[arg(long)]
        stdio: bool,
        /// Directory containing files this MCP process may read. Repeat for additional roots.
        #[arg(long, value_name = "DIR")]
        allow_root: Vec<PathBuf>,
    },
}

#[derive(Debug, clap::Args)]
struct InputArgs {
    /// Path to one explicitly selected UTF-8 text file.
    #[arg(long)]
    input: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(exit_code) => exit_code,
        Err(error) if is_broken_pipe(error.as_ref()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cv-linter: {error}");
            ExitCode::from(3)
        }
    }
}

async fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match cli.command {
        Command::Lint(args) => {
            let input = read_selected_file(&args.input)?;
            let report = lint_plain_text(&input)?;
            let has_errors = report
                .findings
                .iter()
                .any(|finding| finding.severity == Severity::Error);
            write_json(&report)?;
            Ok(if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        Command::ExtractText(args) => {
            let input = read_selected_file(&args.input)?;
            write_json(&extract_plain_text(&input)?)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp {
            stdio: true,
            allow_root,
        } => {
            let service = mcp::CvLinterMcp::new(allow_root)?.serve(stdio()).await?;
            service.waiting().await?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp { stdio: false, .. } => Err("the MVP MCP server requires --stdio".into()),
    }
}

fn write_json(value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = serde_json::to_vec_pretty(value)?;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(&encoded)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

fn is_broken_pipe(error: &(dyn std::error::Error + 'static)) -> bool {
    error
        .downcast_ref::<io::Error>()
        .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
}
