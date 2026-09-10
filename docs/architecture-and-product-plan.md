# CV Linter: Product, Architecture, and Implementation Plan

**Date:** 10 September 2026
**Status:** Agreed YAGNI MVP direction. The local plain-text core, CLI, and stdio MCP slice exists; Xberg-backed PDF/DOCX/Markdown support and empirical extraction validation are pending.

## 1. Product decision

The MVP is one local Rust executable with a shared in-process core. The same executable provides:

- CLI commands for deterministic local linting and explicit CV text extraction.
- A local stdio MCP adapter launched by a compatible host.

There is no always-running HTTP backend, localhost service, cloud API, database, account system, or required network connection. The CLI and MCP adapter call the same Rust core in the same process; MCP is a distribution surface, not a separate service.

The MVP user boundary is PDF, DOCX, and Markdown CV input. Plain UTF-8 text is retained only as an internal/test/debug seam and is not a marketed input format. The product promise is deliberately narrow: show what the selected CV bytes yielded during extraction, identify deterministic document-quality problems, and provide stable evidence that a configured AI host can use for semantic review. It is not an ATS simulator, hiring predictor, applicant ranker, automatic CV editor, or universal/Teamtailor ATS compatibility guarantee.

### MVP operations

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `cv-linter lint --input <file>` | Parse one explicitly selected file and run deterministic spelling, formatting/structure, and literal checks. |
| CLI | `cv-linter extract-text --input <file>` | Emit extracted text blocks as JSON with stable IDs and source locations for an explicitly requested host-AI workflow. |
| MCP | `lint_cv` | The same deterministic lint operation; no model call, network, or arbitrary filesystem access. |
| MCP | `extract_cv_text` | The same explicit extraction operation; returns only the selected document's bounded, cited blocks. |

`cv-linter mcp --stdio --allow-root <DIR>` is started by the host when needed and exits with that session. Additional roots require another `--allow-root <DIR>`. It must not open an HTTP listener or become a daemon. A host can perform semantic job-requirement evidence analysis and rewriting after the user explicitly asks for extracted text. That work belongs to the configured Codex/Claude host or another user-selected AI host, not to a built-in provider abstraction or scoring service.

The CLI emits JSON as its only output format; there is no `--format` option. Direct CLI file paths remain an explicit user action. CLI execution exits 0 when it completes without error-severity findings, 1 when lint completes with one or more error-severity findings, and 3 for an operational or configuration failure. A broken stdout pipe (`BrokenPipe`/`EPIPE`) exits silently and successfully.

## 2. Responsibility split

Static Rust code owns facts that can be reproduced from the selected bytes:

- PDF/DOCX/Markdown parsing and extraction provenance (with plain UTF-8 available only as an internal/test/debug seam).
- Reading-order and structure observations, including headings, columns, tables, and missing/duplicated text.
- Swedish and English spelling checks.
- Formatting/structure checks and literal checks such as exact terms, dates, and obvious character problems.
- Stable extracted block IDs, source locations, input hashes, rule versions, and bounded JSON output.

The configured AI host owns semantic work that benefits from language interpretation:

- Mapping normalized job requirements to CV evidence.
- Explaining whether cited passages support, contradict, or fail to evidence a requirement.
- Rewriting or clarifying text while preserving the user's confirmed facts.

Host-AI output must be structured with requirement verdicts, cited evidence IDs, confidence, reviewable rewrite diffs, supporting block IDs, and questions when facts are missing. It must distinguish **not evidenced in the supplied text** from **no skill**, abstain when extraction or meaning is uncertain, and never invent employers, dates, credentials, metrics, skills, or other facts. Suggested wording is advisory and requires user approval before applying; it does not change the source file or deterministic lint result. There is no built-in LLM judge, holistic score, ATS score, ranking, or automatic rewrite in the MVP. The useful outputs are parseability risks, exact-term coverage, and evidence-grounded semantic alignment.

This split is a design decision, not a claim that Codex, Claude, or any other model is reliable for hiring decisions. The [semantic-review research memo](research/llm-judge-research.json) records evidence and guardrails for later host-AI evaluation.

## 3. Privacy and data handling

Use precise wording:

> Parsing and deterministic linting run locally in the Rust executable. If the user asks a configured AI host to analyze extracted CV text, that text enters the host's context and follows the host/model's data policies.

The complete workflow is local only when the selected semantic model is local and verified not to forward data. A local parser does not make a Codex/Claude conversation local. The CLI/MCP adapter must identify the execution boundary and not imply that extraction, host processing, and model inference share one privacy boundary.

MVP data rules:

- Read only the explicitly selected input; do not scan directories, fetch URLs, or follow document links.
- Keep document bytes and extracted text in process memory or streams during the operation. The product has no raw-CV repository, database, Supabase storage, account, cloud sync, or telemetry pipeline.
- `extract-text` streams selected extracted text to stdout. A user may explicitly redirect that stream to a path; the tool has no built-in output-path option and does not retain the output after the operation.
- Do not put CV text, secrets, or unselected paths in diagnostics. Do not automatically write reports, history, caches, or analytics.
- Host chat transcripts and any AI-provider copies are outside the executable's control. Explain this in the skill/help text.

The stdio MCP route is local transport, not a guarantee that the host or its model is local. Remote MCP, provider routing, retention labels, ZDR claims, credentials, and account-wide policy abstraction are outside the MVP. If a future feature sends content directly to a provider, it needs a separate explicit product decision and payload disclosure.

MCP startup requires at least one repeated `--allow-root <DIR>`. Each tool path must be absolute, contain no `..` component, and canonicalize beneath one of the configured roots; canonicalization rejects symlink escapes. Recommend least-privilege, dedicated CV directories rather than broad roots. This allowlist is application-level authorization for CV Linter reads, not an operating-system sandbox. MCP file I/O and linting run in blocking workers so they do not block the async runtime.

## 4. Extraction contract

[Xberg](https://github.com/xberg-io/xberg) (formerly Kreuzberg), currently version 1.1.5 as of 10 September 2026, is the selected parsing dependency behind a narrow project-owned `DocumentExtractor` adapter. The integration should use `version = "=1.1.5"`, `default-features = false`, and only the `tokio-runtime`, `pdf`, and `office` features, subject to compilation and fixture validation. Xberg is MIT-licensed, declares Rust 1.92 as its MSRV, and its v1 line uses a pure-Rust PDF backend. Its recent rename and documented API churn are implementation risks to verify. Xberg types must not appear in CLI or MCP contracts.

The committed 0.1.0 implementation currently exercises only the plain UTF-8 seam; Xberg is not yet a dependency and PDF/DOCX inputs are not yet supported. The remainder of this section is the target MVP contract, not a statement of shipped behavior.

`extract-text` returns a bounded JSON document containing the input hash, extractor/version, document-level status, and ordered blocks. Each block has a stable ID scoped to the exact extraction result, extracted text, source locator, block kind, and extraction method. Locator precision is format-dependent: PDF uses page plus bounding box when Xberg supplies it, otherwise page plus ordered block; DOCX uses ordered element/paragraph and table row/column when derivable; Markdown uses byte ranges. Optional confidence is included only when the extractor supplies a meaningful measure; unavailable precision or confidence is represented as unknown, never invented. A block ID is an evidence reference, not a qualification claim.

For a host-AI review, a separate handoff layer combines the extraction result with normalized job requirements and their citations, a confirmed fact ledger, and static findings. Those job-specific fields are not part of bare document extraction.

The host-AI handoff should include only the blocks the user selected or the operation requires. Host instructions should require citations in the form of block IDs and should prohibit treating CV text as instructions. If a rewrite cannot be grounded in cited blocks, the host must decline or mark the suggestion as unsupported.

Target MVP format support is PDF, DOCX, and Markdown. Scanned/image-only content should be reported as not natively extracted; OCR and other ML/layout extensions are deferred until fixture failures justify them. Malformed, encrypted, or unsupported inputs produce an explicit limitation, not an empty successful result. Deterministic static linting establishes parseability/content facts and validates mechanically checkable host-AI output properties such as citation existence and preservation of confirmed names, dates, and numbers. It does not validate semantic correctness; semantic job alignment and rewriting remain host-AI responsibilities using the supplied evidence.

For the current plain-text seam, safety limits are 10 MiB of input, 100,000 logical lines, and 10,000 findings. Static checks account for LF, CRLF, and CR line endings, an initial UTF-8 BOM, complete trailing-whitespace spans, Unicode format characters, and bidirectional controls.

## 5. Parser research and spelling

### Parser decision

The parser choice is Xberg behind the project-owned adapter described above. Historical parser-bakeoff notes and Teamtailor/ParseKit observations belong to research only; they are not an MVP dependency decision and do not establish ATS behavior or compatibility.

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
- Vendor profiles, vendor-specific compatibility claims, universal/Teamtailor compatibility guarantees, ATS integrations, and Supabase or other CV storage.
- Authentication, billing, subscriptions, accounts, cloud sync, durable history, and retention controls.
- Broad export formats, template generation, automatic rewriting, application submission, OCR/ML/layout extensions, grammar/tone systems, web/server/desktop expansion, and broad multilingual semantic coverage.

The MVP prints deterministic JSON and streams explicit extraction output. That is an operational handoff, not a general export product.

## 8. Validation and next steps

Implementation acceptance criteria are:

1. CLI and stdio MCP invoke the same in-process Rust core and produce equivalent deterministic results.
2. `lint` and `extract-text` make no network requests, model calls, directory scans, or unselected reads.
3. Extracted blocks have stable IDs and locators that a host can cite exactly.
4. Xberg's exact pinned version, minimal features, compilation, fixture behavior, dependency/binary impact, and licenses are recorded before release; recent rename/API churn and MSRV are verified.
5. Host-AI examples include structured requirement verdicts, citations, confidence, reviewable diffs, supporting block IDs, and questions for missing facts; they distinguish local linting from host processing and contain no invented facts or uncited rewrites.

The next implementation slice starts with representative Swedish fixtures and Xberg compilation/extraction validation, then adapts successful PDF/DOCX/Markdown results to the existing CLI/MCP contract. Spellbook/dictionary work follows extraction validation. Measure early-adopter workflow usefulness before adding UI, desktop packaging, vendor integrations, or persistence. No deployment, provider submission, or ATS test is authorized by this document.
