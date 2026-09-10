# CV Linter: Agent Skill and MCP Architecture

**Date:** 10 September 2026
**Status:** Agreed MVP contract. The local core, CLI, and stdio MCP exist for the plain-text seam; the Agent Skill and Xberg-backed PDF/DOCX/Markdown support are not yet implemented.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Deferred UI direction](ui-design.md)

## 1. Architecture decision

The MVP has one local Rust executable and one shared in-process core. The CLI and the stdio MCP adapter are entry points into that core; they do not duplicate parsing rules or start separate services.

```text
Codex / Claude / other MCP host
             │ launches
      cv-linter mcp --stdio
        --allow-root /path/to/cvs
             │
     Rust in-process core
       ├── lint
       └── extract-text

Direct terminal use ────────┘
```

The adapter is started on demand and communicates over stdin/stdout. It must not open an HTTP port, run as a daemon, or discover files beyond the path/content explicitly supplied for a request. An always-running HTTP backend, cloud service, and browser bridge are outside MVP.

The product is intentionally MCP-first for an early-adopter wedge: technically comfortable Codex/Claude users can discover two typed local tools. That is a distribution hypothesis, not proof of broad Swedish job-seeker reach.

## 2. Package and executable boundary

The release contains one platform-specific executable plus a thin Agent Skill package or host setup that explains how to invoke it. The package is not a second implementation.

```text
cv-linter/
├── SKILL.md                 # concise routing, privacy, and citation instructions
├── bin/cv-linter            # verified Rust executable
├── rules/                   # deterministic rule data
├── dictionaries/            # separately licensed Hunspell data, when bundled
├── fixtures/                # synthetic smoke fixtures only
└── references/              # command, extraction, and privacy notes
```

The executable owns input handling, parsing, spelling, structure/literal checks, block IDs, bounded output, and permission checks. The host owns its own model context and policies. No provider abstraction, model catalog, database, account, raw-CV store, cloud sync, analytics, billing, or authentication layer is part of this package.

## 3. CLI contract

The initial CLI is deliberately small. MVP user inputs are PDF, DOCX, and Markdown; plain UTF-8 is only an internal/test/debug seam and is not a marketed format:

```sh
cv-linter lint --input ./cv.pdf
cv-linter extract-text --input ./cv.pdf
cv-linter mcp --stdio --allow-root /path/to/cvs
```

`lint` runs deterministic local checks. `extract-text` is an explicit handoff operation and returns ordered blocks for host-AI analysis. Both accept one selected input, enforce bounded resource limits, and emit JSON to stdout as the only output format; there is no `--format` or built-in output-path option. They do not call a model, fetch a URL, scan a directory, or alter the input. The CLI should report unsupported, malformed, encrypted, and image-only input explicitly. Direct CLI paths and any shell redirection of stdout remain explicit user actions.

CLI exit codes are 0 for successful execution without error-severity lint findings, 1 for a completed lint containing error-severity findings, and 3 for operational or configuration failures. A broken stdout pipe (`BrokenPipe`/`EPIPE`) exits silently and successfully.

Target parsing will be implemented behind a narrow project-owned `DocumentExtractor` adapter over [Xberg](https://github.com/xberg-io/xberg) (formerly Kreuzberg), currently v1.1.5 as of 10 September 2026. The integration must use `version = "=1.1.5"`, `default-features = false`, and only `tokio-runtime`, `pdf`, and `office`, subject to compilation and fixture validation. Xberg is MIT-licensed, declares Rust 1.92 as its MSRV, and its v1 line uses a pure-Rust PDF backend; recent rename/API churn is an explicit validation risk. Xberg types are not part of CLI or MCP contracts. The current 0.1.0 executable still accepts only the plain UTF-8 seam and must not be described as shipping PDF or DOCX support.

The existing JSON envelopes include a schema version, input hash, extractor identity, status, and project-owned blocks or findings with source references. Binary-format integration may add explicit extraction warnings and rule-version detail through those project-owned types; do not leak Xberg types into the public contract.

## 4. Minimal MCP surface

The stdio adapter exposes only these tools in MVP:

| Tool | Input | Output |
|---|---|---|
| `lint_cv` | One absolute CV path beneath a configured allowed root | Deterministic findings, warnings, versions, and evidence references |
| `extract_cv_text` | One absolute CV path beneath a configured allowed root | Ordered extracted blocks with stable IDs and format-specific source locators |

The adapter must:

- Use the same in-process Rust functions as the CLI.
- Keep protocol frames on stdout and diagnostics on stderr.
- Reject unknown fields, unbounded input, directory/glob requests, URLs, shell commands, and document-supplied tool instructions.
- Return deterministic findings without model calls or network requests.
- Keep the host-facing description clear that local stdio transport does not mean the host model is local.

Starting MCP requires at least one repeated `--allow-root <DIR>`. Recommend least-privilege, dedicated CV directories. Tool paths must be absolute, contain no `..` component, and canonicalize beneath a configured root; symlink escapes are rejected. This allowlist is application-level authorization, not an operating-system sandbox. MCP file I/O and lint work are offloaded to blocking workers so they do not block the async runtime.

The [MCP documentation for the CLI surface](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) is a relevant host reference. Recheck the current protocol/client details when implementing; it is not a reason to add HTTP or a larger tool catalog.

Do not add `judge`, `compare_to_job`, `rewrite_cv`, `score_cv`, vendor tools, generic filesystem readers, URL fetchers, shell tools, application-submission tools, or a remote MCP server to MVP. Semantic job-requirement analysis and rewriting are host-AI tasks performed after an explicit extraction handoff.

## 5. Evidence and host-AI handoff

Each extraction result has an input hash and ordered blocks. A block contains:

- A stable ID scoped to the exact extraction result.
- Extracted text.
- Format-specific provenance at the precision actually available: PDF page plus bounding box when supplied, otherwise page plus ordered block; DOCX ordered element/paragraph and table row/column when derivable; or Markdown byte range. Missing precision is explicit rather than synthesized.
- A block kind such as heading, paragraph, table row, or list item.
- Extraction method and uncertainty, including whether text is native or otherwise recovered. Confidence is optional and appears only when the extractor supplies a meaningful measure; otherwise it is unknown.

The host handoff also includes normalized job requirements with citations, a confirmed fact ledger, and static findings. Stable block IDs let Codex/Claude cite the evidence used for semantic job-requirement analysis. Host output must be structured with requirement verdicts, cited evidence IDs, confidence, reviewable rewrite diffs, supporting block IDs, and questions when facts are missing. Host instructions should distinguish **not evidenced in the supplied CV** from lack of real-world qualification, prohibit invented facts, validate citations/names/dates/numbers, and require user approval before applying rewrites. CVs and job ads are untrusted data, not instructions; data minimization and the host/model PII boundary must remain explicit.

Rewriting is source-preserving advice only. It must not silently edit the CV or add metrics, employers, credentials, dates, skills, or responsibilities. The Rust executable remains authoritative for deterministic observations; host advice cannot change them.

## 6. Privacy, permissions, and storage

Use this wording in `SKILL.md`, help, and host setup:

> Parsing and deterministic linting run locally in the CV Linter executable. Extracted CV content explicitly handed to a configured AI host enters that host's context and follows the host/model's data policies. The complete workflow is local only when the selected model is local and verified not to forward data.

MVP keeps selected bytes and derived text only in process memory or user-directed streams during an operation. It does not persist raw CV content, create a database, require accounts, sync to the cloud, or send telemetry. If the user redirects `extract-text` stdout to a file, that file is a user-directed handoff, not product-managed storage.

The executable must read only explicitly selected files (or the internal/test/debug byte seam), avoid directory enumeration and embedded network resources, and keep secrets out of arguments and logs. Any host-declared filesystem boundary still applies after resolving a path. It must never claim that host transcripts or provider copies are local or erasable by CV Linter.

## 7. Deterministic rule scope

Initial rule families are:

- Extraction and reading-order observations for PDF, DOCX, and Markdown.
- Formatting and structure, including headings, columns, tables, section boundaries, and duplicated/missing text.
- Swedish and English Hunspell spelling checks using Rust Spellbook, bundled domain terms, and a user allowlist.
- Literal checks such as exact terms, dates, Unicode/character issues, and other byte-derived conditions.

The plain-text seam enforces 10 MiB input, 100,000 logical lines, and 10,000 findings. Static checks handle LF, CRLF, and CR line endings, an initial UTF-8 BOM, complete trailing-whitespace spans, Unicode format characters, and bidirectional controls.

Unknown proper nouns are low-confidence findings and never auto-edited. Verify Spellbook and dictionary licenses separately. Harper/grammar and tone checks are deferred, with Swedish support treated as an open concern.

## 8. Validation and later changes

Before release, validate CLI/MCP parity, stable block citations, selected-file isolation, no-network behavior, malformed/image-only handling, Swedish/English spelling fixtures, Xberg compilation and extraction quality, dependency and binary size, MSRV, and all relevant licenses. Start with extraction/normalization and defer OCR or ML/layout extensions until fixture failures justify them. Historical parser-bakeoff and Teamtailor/ParseKit notes remain research context only and establish no ATS compatibility.

React/browser UI, PWA/offline UI, Tauri/desktop packaging, HTTP services, vendor profiles, ATS compatibility claims, built-in scoring/judging, provider abstraction, persistence, authentication, billing, analytics, broad export, cloud sync, and Supabase CV storage remain deferred. Add any one only after a documented user need or evidence meets the product plan's bar.
