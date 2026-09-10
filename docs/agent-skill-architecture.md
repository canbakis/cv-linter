# CV Linter: Agent Skill and MCP Architecture

**Date:** 10 September 2026
**Status:** Agreed MVP contract; documentation only. No executable, skill, or MCP implementation is present.
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

The initial CLI is deliberately small:

```sh
cv-linter lint --input ./cv.pdf --format json
cv-linter extract-text --input ./cv.pdf --format json
cv-linter mcp --stdio
```

`lint` runs deterministic local checks. `extract-text` is an explicit handoff operation and returns ordered blocks for host-AI analysis. Both accept one selected input, bounded resource limits, and an explicit output stream/path. They do not call a model, fetch a URL, scan a directory, or alter the input. The CLI should report unsupported, malformed, encrypted, and image-only input explicitly.

The JSON envelope should include a schema version, input hash, extractor/rule versions, status, warnings, and stable evidence references. Exact field names remain an implementation detail until the first schema is written; this document does not authorize code changes.

## 4. Minimal MCP surface

The stdio adapter exposes only these tools in MVP:

| Tool | Input | Output |
|---|---|---|
| `lint_cv` | One explicit path or supplied bytes, locale, and bounded limits | Deterministic findings, warnings, versions, and evidence references |
| `extract_cv_text` | One explicit path or supplied bytes, locale, and bounded limits | Ordered extracted blocks with stable IDs and source locators |

The adapter must:

- Use the same in-process Rust functions as the CLI.
- Keep protocol frames on stdout and diagnostics on stderr.
- Reject unknown fields, unbounded input, directory/glob requests, URLs, shell commands, and document-supplied tool instructions.
- Return deterministic findings without model calls or network requests.
- Keep the host-facing description clear that local stdio transport does not mean the host model is local.

The [MCP documentation for the CLI surface](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) is a relevant host reference. Recheck the current protocol/client details when implementing; it is not a reason to add HTTP or a larger tool catalog.

Do not add `judge`, `compare_to_job`, `rewrite_cv`, `score_cv`, vendor tools, generic filesystem readers, URL fetchers, shell tools, application-submission tools, or a remote MCP server to MVP. Semantic job-requirement analysis and rewriting are host-AI tasks performed after an explicit extraction handoff.

## 5. Evidence and host-AI handoff

Each extraction result has an input hash and ordered blocks. A block contains:

- A stable ID scoped to the exact extraction result.
- Extracted text.
- A source locator such as page/coordinates, paragraph/table location, or line/offset.
- A block kind such as heading, paragraph, table row, or list item.
- Provenance and uncertainty, including whether text is native or otherwise recovered.

Stable block IDs let Codex/Claude cite the evidence used for semantic job-requirement analysis. Host instructions should require citations for every supported claim and rewrite, distinguish **not evidenced in the supplied CV** from lack of real-world qualification, and prohibit invented facts. If evidence cannot be located or extraction is ambiguous, the host should abstain or ask the user to inspect the source.

Rewriting is source-preserving advice only. It must not silently edit the CV or add metrics, employers, credentials, dates, skills, or responsibilities. The Rust executable remains authoritative for deterministic observations; host advice cannot change them.

## 6. Privacy, permissions, and storage

Use this wording in `SKILL.md`, help, and host setup:

> Parsing and deterministic linting run locally in the CV Linter executable. Extracted CV content explicitly handed to a configured AI host enters that host's context and follows the host/model's data policies. The complete workflow is local only when the selected model is local and verified not to forward data.

MVP keeps selected bytes and derived text only in process memory or user-directed streams during an operation. It does not persist raw CV content, create a database, require accounts, sync to the cloud, or send telemetry. An explicit `extract-text` output path is a user-directed handoff, not product-managed storage.

The executable must read only explicitly selected files or supplied bytes, avoid directory enumeration and embedded network resources, and keep secrets out of arguments and logs. Any host-declared filesystem boundary still applies after resolving a path. It must never claim that host transcripts or provider copies are local or erasable by CV Linter.

## 7. Deterministic rule scope

Initial rule families are:

- Extraction and reading-order observations.
- Formatting and structure, including headings, columns, tables, section boundaries, and duplicated/missing text.
- Swedish and English Hunspell spelling checks using Rust Spellbook, bundled domain terms, and a user allowlist.
- Literal checks such as exact terms, dates, Unicode/character issues, and other byte-derived conditions.

Unknown proper nouns are low-confidence findings and never auto-edited. Verify Spellbook and dictionary licenses separately. Harper/grammar and tone checks are deferred, with Swedish support treated as an open concern.

## 8. Validation and later changes

Before release, validate CLI/MCP parity, stable block citations, selected-file isolation, no-network behavior, malformed/image-only handling, Swedish/English spelling fixtures, parser extraction quality, dependency and binary size, and all relevant licenses. Use a time-boxed bake-off between [excoffierleonard/parser](https://github.com/excoffierleonard/parser) and [upstream ParseKit](https://github.com/scientist-labs/parsekit) on representative Swedish CV fixtures. Evaluate reading order, columns, headings, Unicode, tables, useful source structure, dependencies, binary size, and licensing.

The [Teamtailor/parsekit-bin](https://github.com/Teamtailor/parsekit-bin) repository is forked from upstream ParseKit, not excoffierleonard/parser. Its Teamtailor-specific changes appear to be packaging/build/release changes, not ATS parser or scoring logic. It supports the claim that a published Teamtailor build exists, not a claim of production ATS compatibility. It is a Ruby gem with a Rust extension, and its MuPDF dependency requires AGPL/commercial licensing review.

React/browser UI, PWA/offline UI, Tauri/desktop packaging, HTTP services, vendor profiles, ATS compatibility claims, built-in scoring/judging, provider abstraction, persistence, authentication, billing, analytics, broad export, cloud sync, and Supabase CV storage remain deferred. Add any one only after a documented user need or evidence meets the product plan's bar.
