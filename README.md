# CV Linter

[![CI](https://github.com/canbakis/cv-linter/actions/workflows/ci.yml/badge.svg)](https://github.com/canbakis/cv-linter/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/canbakis/cv-linter?display_name=tag&sort=semver)](https://github.com/canbakis/cv-linter/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Local, deterministic CV extraction and linting for PDF, DOCX, and Markdown,
available as a CLI, an MCP server, and a Claude Desktop extension.

CV Linter checks document structure, extraction quality, suspicious formatting,
and English or Swedish spelling. Results are JSON with source locations, stable
block IDs, and a SHA-256 input identity. It never edits the selected document.

## Install

On macOS or glibc-based Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/canbakis/cv-linter/releases/latest/download/cv-linter-installer.sh | sh
```

On Windows:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/canbakis/cv-linter/releases/latest/download/cv-linter-installer.ps1 | iex"
```

Release archives, checksums, and build attestations are available on the
[releases page](https://github.com/canbakis/cv-linter/releases). Prebuilt Linux
binaries target distributions comparable to Ubuntu 22.04 or newer. On older
glibc or musl systems, install from source:

```sh
cargo install --git https://github.com/canbakis/cv-linter \
  --tag v0.1.0 --locked
```

## Claude Desktop

Download `cv-linter.mcpb` from the latest release, then open Claude Desktop and
choose **Settings → Extensions → Advanced settings → Install Extension…**.

The extension bundles the executable for Apple Silicon and Intel macOS and x64
Windows. It requires no Rust toolchain, API key, or background service. Give
Claude the absolute path to the CV you want reviewed.

## CLI

```sh
cv-linter lint --input resume.pdf
cv-linter extract-text --input resume.docx
cv-linter lint --input resume.md --allow-word ProductName
```

`lint` returns deterministic findings and `extract-text` returns ordered,
source-located text blocks. Run `cv-linter --help` for the complete interface.

## MCP and Claude plugin

Start the local stdio MCP server with:

```sh
cv-linter mcp --stdio
```

It exposes `lint_cv` and `extract_cv_text`. Each call reads only the absolute
file path supplied for that operation. The repository is also a Claude plugin
for Claude Code and Cowork; after installing the executable, test a checkout
with:

```sh
claude --plugin-dir /absolute/path/to/cv-linter
```

## Privacy and scope

The executable performs no network requests, calls no model, scans no
directories, and persists no CV content. If an MCP host sends extracted text to
a model, that text follows the host and model's data policies.

CV Linter is not an ATS simulator, hiring predictor, automatic rewriter, or
compatibility guarantee. Scanned documents without a readable text layer are
reported explicitly; OCR is not included.

## Development

The repository pins Rust 1.94.0. To build and validate locally:

```sh
cargo build
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

See the [architecture and product plan](docs/architecture-and-product-plan.md),
[CLI and MCP architecture](docs/agent-skill-architecture.md), and
[release guide](docs/releasing.md) for the detailed contracts and limitations.
