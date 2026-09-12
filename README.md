# CV Linter

CV Linter is a local Rust executable for deterministic CV extraction and linting. The MVP accepts one explicitly selected PDF, DOCX, or Markdown file at a time. It does not scan directories, make network requests, call a model, or modify the input. A configured host can combine linting with explicit extraction for ATS-oriented risk review and CV best-practice review.

## Install

GitHub Releases are the initial distribution source. After the first tagged
release, macOS and Linux users can install the latest compatible binary with:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/canbakis/cv-linter/releases/latest/download/cv-linter-installer.sh | sh
```

Windows users can install from PowerShell with:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/canbakis/cv-linter/releases/latest/download/cv-linter-installer.ps1 | iex"
```

These installers download the matching binary from the GitHub release and add
it under Cargo's binary directory. Release archives, SHA-256 checksums, and
GitHub build attestations remain available for users who prefer to download and
verify artifacts manually.

Rust users can build directly from a tagged source release:

```sh
cargo install --git https://github.com/canbakis/cv-linter \
  --tag v0.1.0 --locked
```

The prebuilt Linux binaries target glibc-based distributions comparable to
Ubuntu 22.04 or newer. Build from the tagged source on older glibc or musl-based
systems.

## Build from source

Install Rust and Cargo. The repository pins the toolchain in [`rust-toolchain.toml`](rust-toolchain.toml) (Rust 1.94.0, with `rustfmt` and `clippy`), so `rustup` will select it when you run Cargo commands from this directory.

Build and run the test suite:

```sh
cargo build
cargo test
```

## Try the CLI

The repository includes a Swedish Markdown fixture:

```sh
cargo run -- lint --input tests/fixtures/swedish-cv.md
cargo run -- extract-text --input tests/fixtures/swedish-cv.md
```

Both commands print pretty-printed JSON to stdout. `lint` reports deterministic findings and `extract-text` returns ordered text blocks with stable IDs and source locations. The JSON includes a schema version, SHA-256 input identity, extractor identity, status, and operation-specific results. Use a built binary in the same way with `target/debug/cv-linter`.

For a one-run spelling exception, repeat `--allow-word`; it is never persisted or used to edit the CV:

```sh
printf '# Profil\n\nErfarenhet med quuxzorp\n' > /tmp/cv.md
cargo run -- lint --input /tmp/cv.md --allow-word quuxzorp
```

Non-spelling findings and spelling findings are separate in schema `0.3.0` (`findings` and `spelling_findings`). Lowercase unknown words are spelling findings; capitalized unknowns are possible-name or specialist-term spelling findings. A host may verify a term with a term-only authoritative web lookup when needed, then pass confirmed terminology as operation-scoped `allow_words` so it is suppressed from the human-facing report. The lookup is host work; the Rust executable remains offline and deterministic. The bundled Spellbook 0.4.2 checks Swedish and English using pinned LibreOffice `sv_SE` and `en_US` dictionaries; dictionary provenance and license notices are documented in [`dictionaries/README.md`](dictionaries/README.md). No spelling suggestions or automatic edits are produced.

Lint JSON records the normalized allow words under `options.spelling_allow_words`, so two runs with different exceptions remain distinguishable. Stable block IDs remain machine-grounding references; human-facing reports should use page, line, or section-friendly locations when available.

Inputs must have a `.pdf`, `.docx`, `.md`, or `.markdown` extension. PDF and DOCX extraction is provided by the pinned Xberg adapter; Markdown is also supported. An image-only or scanned document may return `status: "no_readable_text"` with an extraction warning. OCR is not included in this MVP, so an image-only CV cannot be linted for text content until it has a readable text layer.

The JSON schema is version `0.3.0`, and the deterministic ruleset is version `0.2.2`. Extraction uses Xberg security limits, including a 30-second cooperative timeout, bounded embedded/archive processing, at most 100 PDF pages, and at most 10,000 output blocks. DOCX has no fixed page count that can be checked without layout. Markdown is capped at 10,000 source lines before extraction; the internal plain-text seam retains its separate 100,000-line limit. Markdown source locations are exact when a complete source line can be conservatively matched to the extracted block; otherwise the locator is explicitly unknown.

Exit codes are:

- `0`: operation completed, and `lint` has no error-severity findings.
- `1`: `lint` completed with at least one error-severity finding.
- `3`: operational or configuration failure (for example, a missing file, unsupported format, malformed document, or invalid MCP setup).

Diagnostics go to stderr; JSON results go to stdout. A broken stdout pipe is treated as a successful exit.

## Stdio MCP

The executable also exposes the same core through two stdio MCP tools: `lint_cv` and `extract_cv_text`. Start it on demand from a compatible MCP host:

```sh
cargo run -- mcp --stdio
```

Each tool call reads only the single absolute document path supplied for that operation and never scans a directory. The MCP adapter does not maintain its own path allowlist, so it can read any supplied file that the operating-system user running it can read. Use it only with a trusted local MCP host and select documents deliberately.

Host configuration syntax varies, so this project documents the launch command rather than prescribing a host-specific configuration file. Configure your host's stdio MCP integration to launch the command above, keeping protocol traffic on stdin/stdout and diagnostics on stderr.

The `lint_cv` tool accepts an operation-scoped `allow_words` array with single-word terms. Values are not persisted, suggestions are not generated, and the executable never auto-edits a CV. For a general CV check, the host should call linting and then explicit extraction, using the extracted evidence for clearly labeled ATS-oriented risk review and CV best-practice review. These reviews must not become an ATS score, pass guarantee, hiring prediction, or vendor-compatibility claim.

Parsing and deterministic linting run locally in the CV Linter executable. If extracted CV text is handed to a configured AI host, it enters that host's context and follows the host/model's data policies. The complete workflow is local only when the selected model is local and verified not to forward data. CV Linter does not persist raw CVs, provide cloud storage, or control host transcripts.

## Claude

Each GitHub release includes `cv-linter.mcpb`, a Claude Desktop extension for
Apple Silicon and Intel macOS and x64 Windows. Download it from the release,
then open Claude Desktop and choose **Settings → Extensions → Advanced
settings → Install Extension…**. The bundle contains all supported binaries and
does not require Rust, an API key, or a background service. Claude Desktop
provides the Node.js launcher runtime. Give Claude the absolute path to the
local PDF, DOCX, or Markdown CV you want reviewed.

The repository is also a Claude plugin for Claude Code and Cowork. Its plugin
combines the [`cv-linter` skill](skills/cv-linter/SKILL.md) with the local MCP
server configuration. Install the `cv-linter` executable first, then test a
checkout with:

```sh
claude --plugin-dir /absolute/path/to/cv-linter
```

The Desktop extension and Claude plugin serve different host surfaces but call
the same two stdio MCP tools. Neither changes the local-processing boundary
described above.

## Scope

The MVP is an evidence-oriented parser and deterministic linter. It is not an ATS simulator, hiring predictor, ranking system, automatic rewriter, or compatibility guarantee for a particular ATS. Semantic job matching and rewriting remain responsibilities of the configured host after an explicit extraction handoff.

The plugin-layout [`skills/cv-linter/SKILL.md`](skills/cv-linter/SKILL.md) is present and validator-clean as host instructions. It routes report generation through a [consistent report template](skills/cv-linter/references/report-template.md), qualitative advice through [bounded CV-writing guidance](skills/cv-linter/references/cv-writing-guidance.md), and configuration questions through the [current and planned rule-configuration boundary](skills/cv-linter/references/rule-configuration.md). The repository includes a Claude plugin manifest, a Claude Desktop MCPB manifest, and tag-driven GitHub release configuration. See the [release guide](docs/releasing.md) for versioning, validation, and the remaining signing limitations.
