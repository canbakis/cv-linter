use clap::{Parser, Subcommand, ValueEnum};
mod mcp;

use cv_linter::{extract_plain_text, lint_plain_text, read_selected_file};
use rmcp::{ServiceExt, transport::stdio};
use serde::Serialize;
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
    },
}

#[derive(Debug, clap::Args)]
struct InputArgs {
    /// Path to one explicitly selected UTF-8 text file.
    #[arg(long)]
    input: PathBuf,
    /// Output format. JSON is the only MVP format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cv-linter: {error}");
            ExitCode::from(3)
        }
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Lint(args) => {
            let input = read_selected_file(&args.input)?;
            write_json(&lint_plain_text(&input)?)?;
        }
        Command::ExtractText(args) => {
            let input = read_selected_file(&args.input)?;
            write_json(&extract_plain_text(&input)?)?;
        }
        Command::Mcp { stdio: true } => {
            let service = mcp::CvLinterMcp.serve(stdio()).await?;
            service.waiting().await?;
        }
        Command::Mcp { stdio: false } => {
            return Err("the MVP MCP server requires --stdio".into());
        }
    }
    Ok(())
}

fn write_json(value: &impl Serialize) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
