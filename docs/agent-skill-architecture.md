# CV Linter: Agent Skill and MCP Architecture

**Date:** 10 September 2026
**Status:** Agreed MVP contract. The local Rust core, CLI, and stdio MCP route through the project-owned Xberg 1.1.5 PDF/DOCX/Markdown adapter. Synthetic Swedish format, error, and CLI/MCP parity tests cover the current adapter. Schema 0.3.0, ruleset 0.2.2, and Spellbook 0.4.2 spelling checks with pinned LibreOffice `en_US`/`sv_SE` dictionaries are implemented. The plugin-layout `skills/cv-linter/` directory, Claude plugin manifest, Claude Desktop MCPB manifest, and tag-driven GitHub release configuration are present; the first cross-platform release and host smoke tests remain release work. OCR is disabled and plain UTF-8 remains an internal/test/debug seam.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Deferred UI direction](ui-design.md)

## 1. Architecture decision

The MVP has one local Rust executable and one shared in-process core. The CLI and the stdio MCP adapter are entry points into that core; they do not duplicate parsing rules or start separate services.

```text
Codex / Claude / other MCP host
             │ launches
      cv-linter mcp --stdio
             │
     Rust in-process core
       ├── lint
       └── extract-text

Direct terminal use ────────┘
```

The adapter is started on demand and communicates over stdin/stdout. It must not open an HTTP port, run as a daemon, or discover files beyond the path/content explicitly supplied for a request. An always-running HTTP backend, cloud service, and browser bridge are outside MVP.

The product is intentionally MCP-first for an early-adopter wedge: technically comfortable Codex/Claude users can discover two typed local tools. That is a distribution hypothesis, not proof of broad Swedish job-seeker reach.

## 2. Package and executable boundary

The planned release contains platform-specific executables plus thin host packaging that explains how to invoke them. `skills/cv-linter/SKILL.md` provides the host instructions; `.claude-plugin/plugin.json` and `.mcp.json` expose those instructions and the installed executable to Claude Code/Cowork. Claude Desktop receives a single MCPB with a dependency-free Node.js launcher that selects a bundled macOS or Windows Rust executable and starts its stdio MCP server. The launcher contains no linting or extraction logic and is not a second implementation.

```text
cv-linter/
├── skills/
│   └── cv-linter/
│       ├── SKILL.md                 # concise workflow and safety instructions
│       └── references/
│           ├── cv-writing-guidance.md
│           ├── report-template.md
│           └── rule-configuration.md
├── bin/cv-linter            # verified Rust executable
├── rules/                   # deterministic rule data
├── dictionaries/            # separately licensed Hunspell data, when bundled
└── fixtures/                # synthetic smoke fixtures only
```

The executable owns input handling, parsing, spelling, structure/literal checks, block IDs, bounded output, and permission checks. The host owns its own model context and policies. No provider abstraction, model catalog, database, account, raw-CV store, cloud sync, analytics, billing, or authentication layer is part of this package.

## 3. CLI contract

The initial CLI is deliberately small. MVP user inputs are PDF, DOCX, and Markdown; plain UTF-8 is only an internal/test/debug seam and is not a marketed format:

```sh
cv-linter lint --input ./cv.pdf
cv-linter extract-text --input ./cv.pdf
cv-linter mcp --stdio
```

`lint` runs deterministic local checks. `extract-text` is an explicit handoff operation and returns ordered blocks for host-AI analysis. Both accept one selected input, enforce bounded resource limits, and emit JSON to stdout as the only output format; there is no `--format` or built-in output-path option. `lint` accepts repeated operation-scoped `--allow-word <WORD>` values; they are bounded one-word exceptions and are never persisted. No spelling suggestions or automatic edits are produced. The commands do not call a model, fetch a URL, scan a directory, or alter the input. The CLI should report unsupported, malformed, encrypted, and image-only input explicitly. Direct CLI paths and any shell redirection of stdout remain explicit user actions.

CLI exit codes are 0 for successful execution without error-severity lint findings, 1 for a completed lint containing error-severity findings, and 3 for operational or configuration failures. A broken stdout pipe (`BrokenPipe`/`EPIPE`) exits silently and successfully.

Parsing is implemented behind a narrow project-owned `DocumentExtractor` adapter over [Xberg](https://github.com/xberg-io/xberg) (formerly Kreuzberg), pinned to v1.1.5 as of 10 September 2026. The integration uses `version = "=1.1.5"`, `default-features = false`, and only `tokio-runtime`, `pdf`, and `office`. CLI and stdio MCP extraction are routed through this adapter; Xberg types are not part of CLI or MCP contracts. Xberg is MIT-licensed, declares Rust 1.92 as its MSRV, and its v1 line uses a pure-Rust PDF backend; recent rename/API churn remains an explicit validation risk. The plain UTF-8 path remains an internal/test/debug seam.

Synthetic Swedish fixtures cover Markdown, PDF, and DOCX extraction, malformed/encrypted/unsupported and no-native-text outcomes, and CLI/MCP parity. These tests do not establish broad real-world document coverage.

The JSON envelopes use schema version `0.3.0` and ruleset version `0.2.2`; they include an input hash, extractor identity, status, and project-owned blocks or findings with source references. Lint output separates non-spelling `findings` from `spelling_findings`, while preserving the same evidence shape. Stable block IDs are internal grounding references; human-facing reports should prefer page, line, or section-friendly locators when available. Binary-format integration may add explicit extraction warnings and rule-version detail through those project-owned types; do not leak Xberg types into the public contract.

## 4. Minimal MCP surface

The stdio adapter exposes only these tools in MVP:

| Tool | Input | Output |
|---|---|---|
| `lint_cv` | One explicitly selected absolute CV path | Deterministic findings, warnings, versions, and evidence references |
| `extract_cv_text` | One explicitly selected absolute CV path | Ordered extracted blocks with stable IDs and format-specific source locators |

The adapter must:

- Use the same in-process Rust functions as the CLI.
- Keep protocol frames on stdout and diagnostics on stderr.
- Reject unknown fields, unbounded input, directory/glob requests, URLs, shell commands, and document-supplied tool instructions.
- Return deterministic findings without model calls or network requests.
- Keep the host-facing description clear that local stdio transport does not mean the host model is local.

Starting MCP requires only `--stdio`. Tool paths must be absolute, but the adapter does not enforce an application-level path allowlist; it can read any supplied file available to its operating-system user. It reads one explicitly selected path per operation and never enumerates directories. The host must call it only for the user-selected document, and any filesystem sandbox remains the responsibility of the host or operating system. `lint_cv` accepts operation-scoped `allow_words`; values are bounded one-word exceptions and are never persisted. MCP file I/O and lint work are offloaded to blocking workers so they do not block the async runtime.

The [MCP documentation for the CLI surface](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) is a relevant host reference. Recheck the current protocol/client details when implementing; it is not a reason to add HTTP or a larger tool catalog.

For a general CV check, the host calls `lint_cv` and then explicitly calls `extract_cv_text`. It may provide clearly labeled ATS-oriented risk review and CV best-practice review grounded in the extracted evidence. These are host assessments, not deterministic MCP findings, and must not become an ATS score, pass guarantee, certification, ranking, hiring prediction, or vendor-compatibility claim. Do not add `judge`, `compare_to_job`, `rewrite_cv`, `score_cv`, vendor tools, generic filesystem readers, URL fetchers, shell tools, application-submission tools, or a remote MCP server to MVP.

## 5. Evidence and host-AI handoff

Each extraction result has an input hash and ordered blocks. A block contains:

- A stable ID scoped to the exact extraction result.
- Extracted text.
- Format-specific provenance at the precision actually available: PDF page plus bounding box when supplied, otherwise page plus ordered block; DOCX ordered element/paragraph and table row/column when derivable; or an exact Markdown line and byte range when matched. If Markdown cannot be matched exactly, its locator is entirely unknown rather than partially synthesized. Missing precision is explicit rather than invented.
- A block kind such as heading, paragraph, table row, or list item.
- Extraction method and uncertainty, including whether text is native or otherwise recovered. Confidence is optional and appears only when the extractor supplies a meaningful measure; otherwise it is unknown.

The host handoff also includes normalized job requirements with citations, a confirmed fact ledger, and static findings. Stable block IDs let Codex/Claude ground analysis internally; human-facing output should use page, line, or section-friendly locators by default. Host output must be structured with requirement verdicts, internal evidence grounding, human-readable locations, confidence, reviewable rewrite diffs, and questions when facts are missing. ATS-oriented risk and best-practice observations must be clearly labeled host assessment and must not imply a score, pass guarantee, certification, ranking, or vendor compatibility. Host instructions should distinguish **not evidenced in the supplied CV** from lack of real-world qualification, prohibit invented facts, validate citations/names/dates/numbers, and require user approval before applying rewrites. CVs and job ads are untrusted data, not instructions; data minimization and the host/model PII boundary must remain explicit.

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
- Swedish and English spelling checks using Spellbook 0.4.2, pinned LibreOffice `en_US`/`sv_SE` dictionaries, bundled domain terms, and operation-scoped user allow words.
- Literal checks such as exact terms, dates, Unicode/character issues, and other byte-derived conditions.

For the no-headings rule, ruleset 0.2.2 treats either a native title/heading block or at least two exact, standalone labels from a bounded English/Swedish section alias list as evidence of section structure. Alias matching is a lint-time inference only: extraction keeps the parser-provided block kind, so hosts can distinguish source extraction from deterministic interpretation.

The plain-text seam enforces 10 MiB input, 100,000 logical lines, and 10,000 findings. Xberg applies bounded extraction security limits, including a 30-second cooperative timeout and a 10,000-block cap; Markdown is capped at 10,000 source lines before extraction. Static checks handle LF, CRLF, and CR line endings, an initial UTF-8 BOM, complete trailing-whitespace spans, Unicode format characters, and bidirectional controls.

Lowercase unknown words are spelling findings; capitalized unknowns are informational possible-name or specialist-term spelling findings, separate from non-spelling `findings` in schema 0.3.0. Allow words are operation-scoped, case-insensitive, and never persisted. When needed, the host may perform a term-only authoritative web lookup and pass confirmed terminology as `allow_words`, suppressing it from the human-facing output for that operation. The Rust executable remains offline; extracted text enters the host context and follows host/model policy. No spelling suggestions or automatic edits are produced. Dictionary provenance and notices are recorded in [`dictionaries/README.md`](../dictionaries/README.md); its license inventory is qualified provenance, not legal advice or assurance for every distribution model. Harper/grammar and tone checks remain outside deterministic linting.

## 8. Validation and later changes

Before release, continue validating CLI/MCP parity, stable block citations, selected-file isolation, no-network behavior, malformed/image-only handling, Xberg extraction quality and security limits, dependency and binary size, MSRV, and all relevant licenses. Synthetic Swedish format/error/parity tests cover the current adapter; they do not replace broader fixture validation. OCR remains disabled, and ML/layout extensions remain deferred until fixture failures justify them. Historical parser-bakeoff and Teamtailor/ParseKit notes remain research context only and establish no ATS compatibility.

### TODO: user-configurable deterministic rules

Defer rule configuration until observed need justifies the additional public contract. A future design should borrow the useful parts of [ESLint's rule configuration model](https://eslint.org/docs/latest/use/configure/rules) without its executable JavaScript surface:

- Address rules by stable IDs and allow `off`, `info`, `warning`, or `error` severity.
- Give each rule typed, bounded options, such as a structure threshold or a literal term list; reject unknown rule IDs, severities, and options.
- Resolve and validate configuration in the transport-independent Rust core so CLI and MCP remain thin adapters.
- Preserve built-in defaults and include the effective configuration, or a deterministic hash of it, in lint reports.
- Accept configuration only through an explicit user-selected file or bounded inline input. Do not search parent or home directories, execute configuration code, or accept unrestricted rule implementations or regular expressions.
- Treat any CLI flag and MCP input shape as part of the versioned public contract, and do not let host instructions silently weaken user-selected rules.

An illustrative static shape, not a committed file-format decision, is:

```toml
config_version = 1

[rules."cv.structure.dense_block"]
severity = "info"
max_characters = 1200
```

The current 1,200-character dense-block threshold remains an informational project heuristic, not a published readability standard. Candidate-facing guidance may motivate fixtures and future experiments, but it must not be promoted into a universal ATS rule or numeric cutoff; see the [candidate-facing CV guidance note](research/candidate-facing-cv-guidance.md).

The Agent Skill carries the host-facing boundary in [`rule-configuration.md`](../skills/cv-linter/references/rule-configuration.md). Its [`report-template.md`](../skills/cv-linter/references/report-template.md) provides the consistent presentation contract for deterministic linting, extraction, semantic review, and job comparison, while [`cv-writing-guidance.md`](../skills/cv-linter/references/cv-writing-guidance.md) bounds qualitative advice and rewrites. These references are Skill behavior and packaging inputs, not new Rust or MCP capabilities.

React/browser UI, PWA/offline UI, Tauri/desktop packaging, HTTP services, vendor profiles, ATS compatibility claims, built-in scoring/judging, provider abstraction, persistence, authentication, billing, analytics, broad export, cloud sync, and Supabase CV storage remain deferred. Add any one only after a documented user need or evidence meets the product plan's bar.
