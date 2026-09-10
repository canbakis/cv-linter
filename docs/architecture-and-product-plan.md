# CV Linter: Product, Architecture, and Implementation Plan

**Date:** 10 September 2026
**Status:** Agreed YAGNI MVP direction; documentation only. No implementation or empirical compatibility result is claimed.

## 1. Product decision

The MVP is one local Rust executable with a shared in-process core. The same executable provides:

- CLI commands for deterministic local linting and explicit CV text extraction.
- A local stdio MCP adapter launched by a compatible host.

There is no always-running HTTP backend, localhost service, cloud API, database, account system, or required network connection. The CLI and MCP adapter call the same Rust core in the same process; MCP is a distribution surface, not a separate service.

The product promise is deliberately narrow: show what the selected CV bytes yielded during extraction, identify deterministic document-quality problems, and provide stable evidence that a configured AI host can use for semantic review. It is not an ATS simulator, hiring predictor, applicant ranker, or automatic CV editor.

### MVP operations

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `cv-linter lint --input <file>` | Parse one explicitly selected file and run deterministic spelling, formatting/structure, and literal checks. |
| CLI | `cv-linter extract-text --input <file> --format json` | Emit extracted text blocks with stable IDs and source locations for an explicitly requested host-AI workflow. |
| MCP | `lint_cv` | The same deterministic lint operation; no model call, network, or arbitrary filesystem access. |
| MCP | `extract_cv_text` | The same explicit extraction operation; returns only the selected document's bounded, cited blocks. |

`cv-linter mcp --stdio` is started by the host when needed and exits with that session. It must not open an HTTP listener or become a daemon. A host can perform semantic job-requirement evidence analysis and rewriting after the user explicitly asks for extracted text. That work belongs to the configured Codex/Claude host or another user-selected AI host, not to a built-in provider abstraction or scoring service.

## 2. Responsibility split

Static Rust code owns facts that can be reproduced from the selected bytes:

- PDF/DOCX/plain-text parsing and extraction provenance.
- Reading-order and structure observations, including headings, columns, tables, and missing/duplicated text.
- Swedish and English spelling checks.
- Formatting/structure checks and literal checks such as exact terms, dates, and obvious character problems.
- Stable extracted block IDs, source locations, input hashes, rule versions, and bounded JSON output.

The configured AI host owns semantic work that benefits from language interpretation:

- Mapping job requirements to CV evidence.
- Explaining whether a cited passage supports a requirement.
- Rewriting or clarifying text while preserving the user's facts.

Host-AI output must cite stable extracted block IDs. It must say **not evidenced in the supplied text** when support is absent, abstain when extraction or meaning is uncertain, and never invent employers, dates, credentials, metrics, skills, or other facts. Suggested wording is advisory; it does not change the source file or deterministic lint result. There is no built-in LLM judge, holistic score, ATS score, ranking, or automatic rewrite in the MVP.

This split is a design decision, not a claim that Codex, Claude, or any other model is reliable for hiring decisions. The [semantic-review research memo](research/llm-judge-research.json) records evidence and guardrails for later host-AI evaluation.

## 3. Privacy and data handling

Use precise wording:

> Parsing and deterministic linting run locally in the Rust executable. If the user asks a configured AI host to analyze extracted CV text, that text enters the host's context and follows the host/model's data policies.

The complete workflow is local only when the selected semantic model is local and verified not to forward data. A local parser does not make a Codex/Claude conversation local. The CLI/MCP adapter must identify the execution boundary and not imply that extraction, host processing, and model inference share one privacy boundary.

MVP data rules:

- Read only the explicitly selected input; do not scan directories, fetch URLs, or follow document links.
- Keep document bytes and extracted text in process memory or streams during the operation. The product has no raw-CV repository, database, Supabase storage, account, cloud sync, or telemetry pipeline.
- `extract-text` may stream selected extracted text to stdout or a path explicitly chosen by the user because that is the requested handoff. The tool does not retain that output after the operation.
- Do not put CV text, secrets, or unselected paths in diagnostics. Do not automatically write reports, history, caches, or analytics.
- Host chat transcripts and any AI-provider copies are outside the executable's control. Explain this in the skill/help text.

The stdio MCP route is local transport, not a guarantee that the host or its model is local. Remote MCP, provider routing, retention labels, ZDR claims, credentials, and account-wide policy abstraction are outside the MVP. If a future feature sends content directly to a provider, it needs a separate explicit product decision and payload disclosure.

## 4. Extraction contract

`extract-text` returns a bounded JSON document containing the input hash, extractor/version, document-level status, and ordered blocks. Each block has a stable ID scoped to the exact extraction result, extracted text, source locator, block kind, and provenance such as native text or OCR if OCR is ever added. A block ID is an evidence reference, not a qualification claim.

The host-AI handoff should include only the blocks the user selected or the operation requires. Host instructions should require citations in the form of block IDs and should prohibit treating CV text as instructions. If a rewrite cannot be grounded in cited blocks, the host must decline or mark the suggestion as unsupported.

Initial format support and limits are implementation decisions to validate with fixtures. Scanned/image-only content should be reported as not natively extracted; OCR is deferred until a measured need exists. Malformed, encrypted, or unsupported inputs produce an explicit limitation, not an empty successful result.

## 5. Parser research and spelling

### Time-boxed extraction bake-off

Before locking the extraction implementation, run a time-boxed spike comparing:

- [excoffierleonard/parser](https://github.com/excoffierleonard/parser)
- [upstream ParseKit](https://github.com/scientist-labs/parsekit)

Use representative Swedish CV fixtures, with controlled examples for single and multi-column layouts, headings, Unicode and Swedish characters, tables, lists, and source-structure preservation. Evaluate reading order, columns, headings, Unicode, tables, ability to preserve useful source structure, dependency footprint, binary size, runtime/resource behavior, and library/data licensing. Record fixture IDs, versions, results, limitations, and license review; do not infer ATS behavior from this bake-off.

The [Teamtailor/parsekit-bin](https://github.com/Teamtailor/parsekit-bin) project is a separate research signal. It is forked from `scientist-labs/parsekit`, not from `excoffierleonard/parser`. Its Teamtailor-specific diff appears to concern packaging, build, and release changes rather than ATS-specific parser or scoring logic. It is evidence that a Teamtailor-published build exists, not evidence of production Teamtailor compatibility. `parsekit-bin` is a Ruby gem with a Rust extension; its MuPDF dependency raises AGPL/commercial licensing questions that must be reviewed separately before adoption or redistribution.

### Spelling

Evaluate [Rust Spellbook](https://github.com/helix-editor/spellbook) with Swedish and English Hunspell dictionaries. Add bundled domain allowlists and a user allowlist. Unknown proper nouns are low-confidence findings only: do not auto-edit or silently add them to an allowlist. Verify the Spellbook/library license and each dictionary's license independently.

Harper-style grammar checking, tone advice, and broad rewriting are later experiments, especially because Swedish grammar support is a material uncertainty. They are not hidden requirements of the deterministic linter.

## 6. Problem, market, and ATS hypotheses

These are hypotheses to validate, not product facts:

- CV authors may benefit from seeing extraction order, structure loss, spelling issues, and literal omissions before submitting a document.
- Swedish CVs may expose Unicode, language, and structure cases that generic English fixtures miss.
- An MCP-first workflow may be a useful early-adopter wedge for technically comfortable Codex/Claude users.
- MCP-first adoption is not evidence of broad reach among Swedish job seekers; non-agent demand, conversion, and willingness to install a local tool remain unknown.
- ATS behavior is configurable by vendor, edition, tenant, workflow, parser version, and employer settings. A local parser result cannot establish how an ATS will score, reject, or rank a CV.

Use the [ATS research memo](research/ats-vendor-research.json) and [authorized-testing plan](research/vendor-evidence-and-testing-plan.md) as research inputs. Preserve the distinction between official documentation, a local observation, and an authorized vendor observation. Candidate-facing articles and anecdotes are hypothesis sources, not authority for compatibility rules.

Do not claim Teamtailor compatibility. In particular, the parsekit-bin repository, a Teamtailor sandbox, a successful upload, or a local extraction result does not justify “Teamtailor-compatible,” “ATS certified,” or a hiring-outcome claim.

## 7. Explicitly deferred scope

Defer until observed user need or evidence justifies the added complexity:

- React/browser UI, PWA/offline UI, and browser inference.
- Tauri or other desktop packaging; require paying-customer evidence before pursuing it.
- Always-running HTTP backends, cloud services, localhost servers, and remote MCP.
- Provider abstraction, provider policy/ZDR catalogs, built-in judge/scoring systems, rubrics, applicant ranking, and analytics.
- Vendor profiles, vendor-specific compatibility claims, ATS integrations, and Supabase or other CV storage.
- Authentication, billing, subscriptions, accounts, cloud sync, durable history, and retention controls.
- Broad export formats, template generation, automatic rewriting, application submission, OCR, grammar/tone systems, and broad multilingual semantic coverage.

The MVP can print deterministic JSON and stream explicit extraction output. That is an operational handoff, not a general export product.

## 8. Validation and next steps

Documentation-only acceptance criteria for the implementation are:

1. CLI and stdio MCP invoke the same in-process Rust core and produce equivalent deterministic results.
2. `lint` and `extract-text` make no network requests, model calls, directory scans, or unselected reads.
3. Extracted blocks have stable IDs and locators that a host can cite exactly.
4. Swedish/English spelling behavior, parser choices, dependencies, binary size, and licenses are recorded from the bake-offs rather than assumed.
5. Host-AI examples distinguish local linting from host processing and contain no invented facts or uncited rewrites.

When implementation is authorized, start with representative Swedish fixtures, the parser bake-off, Spellbook/dictionary license review, the two CLI operations, and the two MCP tools. Measure early-adopter workflow usefulness before adding UI, desktop packaging, vendor integrations, or persistence. No commit, deployment, provider submission, or ATS test is authorized by this document.
