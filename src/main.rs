use clap::{Parser, Subcommand, error::ErrorKind};
mod mcp;

use cv_linter::{Severity, extract_document, lint_document_with_allow_words, read_selected_file};
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
    /// Run deterministic checks on one PDF, DOCX, or Markdown CV.
    Lint(LintArgs),
    /// Extract ordered, source-located text blocks from one PDF, DOCX, or Markdown CV.
    ExtractText(InputArgs),
    /// Start the local MCP server.
    Mcp {
        /// Serve MCP over standard input and output.
        #[arg(long)]
        stdio: bool,
    },
}

#[derive(Debug, clap::Args)]
struct InputArgs {
    /// Path to one explicitly selected PDF, DOCX, or Markdown file.
    #[arg(long)]
    input: PathBuf,
}

#[derive(Debug, clap::Args)]
struct LintArgs {
    /// Path to one explicitly selected PDF, DOCX, or Markdown file.
    #[arg(long)]
    input: PathBuf,
    /// Accept one spelling term for this run. Repeat for additional terms.
    #[arg(long = "allow-word", value_name = "WORD")]
    allow_words: Vec<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(3)
            };
            return match error.print() {
                Ok(()) => exit_code,
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("cv-linter: {error}");
                    ExitCode::from(3)
                }
            };
        }
    };

    match run(cli).await {
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
            let report =
                lint_document_with_allow_words(&args.input, &input, &args.allow_words).await?;
            let has_errors = report
                .findings
                .iter()
                .chain(&report.spelling_findings)
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
            write_json(&extract_document(&args.input, &input).await?)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp { stdio: true } => {
            let service = mcp::CvLinterMcp::new().serve(stdio()).await?;
            service.waiting().await?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp { stdio: false } => Err("the MVP MCP server requires --stdio".into()),
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
