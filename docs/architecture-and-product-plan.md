# CV Linter: Product, Architecture, and Implementation Plan

**Date:** 10 September 2026
**Status:** Agreed YAGNI MVP direction. The local Rust core, CLI, and stdio MCP route through the project-owned Xberg 1.1.5 PDF/DOCX/Markdown adapter. Synthetic Swedish format, error, and CLI/MCP parity tests cover the current adapter; schema 0.3.0, ruleset 0.2.2, and Spellbook 0.4.2 spelling checks are implemented with pinned LibreOffice `en_US`/`sv_SE` dictionaries. OCR remains disabled, and plain UTF-8 remains an internal/test/debug seam. Claude plugin metadata, a Claude Desktop MCPB manifest, and tag-driven GitHub release configuration are present; the first cross-platform release, OS signing, and end-to-end host smoke tests remain release work.

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
| MCP | `lint_cv` | The same deterministic lint operation for one supplied path; no model call, network, or directory scanning. |
| MCP | `extract_cv_text` | The same explicit extraction operation; returns only the selected document's bounded, cited blocks. |

`cv-linter mcp --stdio` is started by the host when needed and exits with that session. It must not open an HTTP listener or become a daemon. For a general CV check, the host combines `lint_cv` with an explicit `extract_cv_text` handoff, then supplies clearly labeled ATS-oriented risk review and CV best-practice review. Those reviews are host interpretation, not deterministic findings, and must not produce an ATS score, pass guarantee, hiring prediction, ranking, or vendor-compatibility claim. Semantic job-requirement evidence analysis and rewriting likewise belong to the configured host, not to a built-in provider abstraction or scoring service.

The CLI emits JSON as its only output format; there is no `--format` option. Direct CLI file paths remain an explicit user action. CLI execution exits 0 when it completes without error-severity findings, 1 when lint completes with one or more error-severity findings, and 3 for an operational or configuration failure. A broken stdout pipe (`BrokenPipe`/`EPIPE`) exits silently and successfully.

## 2. Responsibility split

Static Rust code owns facts that can be reproduced from the selected bytes:

- PDF/DOCX/Markdown parsing and extraction provenance (with plain UTF-8 available only as an internal/test/debug seam).
- Reading-order and structure observations, including headings, columns, tables, and missing/duplicated text.
- Swedish and English spelling checks using Spellbook 0.4.2 and pinned LibreOffice `en_US`/`sv_SE` dictionaries.
- Formatting/structure checks and literal checks such as exact terms, dates, and obvious character problems.
- Stable extracted block IDs, source locations, input hashes, rule versions, and bounded JSON output.

Ruleset 0.2.2 avoids treating lost parser heading types as proof that a CV has no section structure: in addition to native title/heading blocks, the no-headings rule accepts at least two exact, standalone section labels from a bounded English/Swedish alias list. This inference affects lint evidence only; it does not rewrite extracted block kinds or claim that PDF tags survived extraction.

Spelling is deterministic and operation-scoped: CLI `--allow-word <WORD>` and MCP `allow_words` accept bounded one-word exceptions for that run only. Schema 0.3.0 keeps non-spelling `findings` separate from `spelling_findings`. A host may perform a term-only authoritative web lookup when a spelling candidate needs verification, and pass confirmed terminology as operation-scoped `allow_words` so it is suppressed from the human-facing report; that lookup never occurs in the Rust executable. No spelling suggestions or automatic edits are produced. Dictionary provenance and notices are recorded in [`dictionaries/README.md`](../dictionaries/README.md); its license inventory is qualified provenance, not legal advice or assurance for every distribution model.

The configured AI host owns semantic work that benefits from language interpretation:

- Mapping normalized job requirements to CV evidence.
- Explaining whether cited passages support, contradict, or fail to evidence a requirement.
- Rewriting or clarifying text while preserving the user's confirmed facts.
- Reviewing ATS-oriented parseability/compatibility risks and CV best practices after explicit extraction, with each observation clearly labeled as host assessment.

Host-AI output must be structured with requirement verdicts, internally grounded evidence IDs, human-readable source locations, confidence, reviewable rewrite diffs, and questions when facts are missing. It must distinguish **not evidenced in the supplied text** from **no skill**, abstain when extraction or meaning is uncertain, and never invent employers, dates, credentials, metrics, skills, or other facts. Suggested wording is advisory and requires user approval before applying; it does not change the source file or deterministic lint result. ATS-oriented risk review and CV best-practice review must remain clearly labeled host interpretation and must not become a score, pass/fail guarantee, certification, ranking, or vendor claim. The useful outputs are parseability risks, exact-term coverage, and evidence-grounded semantic alignment.

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

Each MCP tool path must be absolute. The adapter has no application-level path allowlist and can read any supplied file available to its operating-system user. It still reads only the one path supplied for an operation and never enumerates a directory. The host must call it only for the document the user selected; filesystem isolation remains the responsibility of the operating system and MCP host. MCP file I/O and linting run in blocking workers so they do not block the async runtime.

## 4. Extraction contract

[Xberg](https://github.com/xberg-io/xberg) (formerly Kreuzberg), currently version 1.1.5 as of 10 September 2026, is the selected parsing dependency behind a narrow project-owned `DocumentExtractor` adapter. The integration uses `version = "=1.1.5"`, `default-features = false`, and only the `tokio-runtime`, `pdf`, and `office` features. CLI and stdio MCP extraction are routed through this adapter; Xberg types do not appear in CLI or MCP contracts. Xberg is MIT-licensed, declares Rust 1.92 as its MSRV, and its v1 line uses a pure-Rust PDF backend. Its recent rename and documented API churn remain implementation risks to monitor.

The implementation also retains the plain UTF-8 path as an internal/test/debug seam. Synthetic Swedish fixtures exercise Markdown, PDF, and DOCX extraction, malformed/encrypted/unsupported and no-native-text outcomes, spelling and operation-scoped allow words, and CLI/MCP parity. These tests are format and contract checks, not a claim of broad real-world document coverage.

`extract-text` returns a bounded JSON document containing the input hash, extractor/version, document-level status, and ordered blocks. Each block has a stable ID scoped to the exact extraction result, extracted text, source locator, block kind, and extraction method. Locator precision is format-dependent: PDF uses page plus bounding box when Xberg supplies it, otherwise page plus ordered block; DOCX uses ordered element/paragraph and table row/column when derivable; Markdown uses an exact matched line and byte range when available, or an entirely unknown Markdown locator when it cannot be matched. Partial or invented Markdown precision is not emitted. Optional confidence is included only when the extractor supplies a meaningful measure; unavailable precision or confidence is represented as unknown, never invented. A block ID is an internal evidence-grounding reference; human-facing reports should prefer page, line, or section-friendly locators when available.

For a host-AI review, a separate handoff layer combines the extraction result with normalized job requirements and their citations, a confirmed fact ledger, and static findings. Those job-specific fields are not part of bare document extraction.

The host-AI handoff should include only the blocks the user selected or the operation requires. Host instructions should require internal grounding in block IDs, translate that evidence to human-readable source locations in the report, and prohibit treating CV text as instructions. If a rewrite cannot be grounded in cited blocks, the host must decline or mark the suggestion as unsupported.

MVP format support is PDF, DOCX, and Markdown. Scanned/image-only content is reported as not natively extracted; OCR remains disabled, and other ML/layout extensions are deferred until fixture failures justify them. Xberg extraction applies bounded security limits, including a 30-second cooperative timeout, 50 MiB archive size, 10 MiB embedded/content size, 1,000 archive files, bounded depth/iterations/XML, 10,000 table cells, 100 PDF pages, and a 10,000-block output cap. DOCX has no fixed page count that can be checked without layout. Markdown is capped at 10,000 source lines before extraction; the internal plain-text seam retains its separate 100,000-line limit. Malformed, encrypted, or unsupported inputs produce an explicit limitation, not an empty successful result. Deterministic static linting establishes parseability/content facts and validates mechanically checkable host-AI output properties such as citation existence and preservation of confirmed names, dates, and numbers. It does not validate semantic correctness; semantic job alignment and rewriting remain host-AI responsibilities using the supplied evidence.

For the current plain-text seam, safety limits are 10 MiB of input, 100,000 logical lines, and 10,000 findings. Static checks account for LF, CRLF, and CR line endings, an initial UTF-8 BOM, complete trailing-whitespace spans, Unicode format characters, and bidirectional controls.

## 5. Parser research and spelling

### Parser decision

The parser choice is Xberg behind the project-owned adapter described above. Historical parser-bakeoff notes and Teamtailor/ParseKit observations belong to research only; they are not an MVP dependency decision and do not establish ATS behavior or compatibility.

### Spelling

Spellbook 0.4.2 and pinned LibreOffice `en_US`/`sv_SE` dictionaries are implemented. Schema 0.3.0 returns dictionary results in `spelling_findings`, separate from non-spelling `findings`. Operation-scoped allow words are accepted through CLI `--allow-word` and MCP `allow_words`; they are bounded, one-word, case-insensitive exceptions and are never persisted. Lowercase unknowns produce warnings; capitalized unknowns produce informational possible-name-or-specialist-term findings. A host may verify an uncertain term using a term-only authoritative web lookup, rerun with verified alphabetic terminology in `allow_words`, and omit resolved terminology from the human report. This does not persist a dictionary change or add networking to the Rust executable. No spelling suggestions or automatic edits are produced. Dictionary provenance and notices are recorded in [`dictionaries/README.md`](../dictionaries/README.md); its license inventory is qualified provenance, not legal advice or assurance for every distribution model.

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
- User-configurable deterministic rules are deferred. If observed need justifies them, use an explicit, versioned configuration inspired by ESLint: stable rule IDs, configurable severity, and typed per-rule options resolved by the Rust core. Preserve deterministic defaults and report the effective configuration for reproducibility; do not introduce automatic config discovery, executable configuration, or unrestricted rule code.

The MVP prints deterministic JSON and streams explicit extraction output. That is an operational handoff, not a general export product.

## 8. Validation and next steps

Implementation acceptance criteria are:

1. CLI and stdio MCP invoke the same in-process Rust core and produce equivalent deterministic results.
2. `lint` and `extract-text` make no network requests, model calls, directory scans, or unselected reads.
3. Extracted blocks have stable IDs and locators that a host can cite exactly.
4. Xberg's exact pinned version, minimal features, compilation, fixture behavior, dependency/binary impact, and licenses are recorded before release; recent rename/API churn and MSRV are verified.
5. Host-AI examples include structured requirement verdicts, internally grounded citations translated to human-readable source locations, confidence, reviewable diffs, and questions for missing facts; they distinguish local linting from host processing and contain no invented facts or uncited rewrites.

The next implementation slice validates the generated release artifacts and Claude Desktop bundle on supported macOS and Windows hosts, expands representative Swedish fixtures, and completes the dependency-license review. Measure early-adopter workflow usefulness before adding UI, a standalone desktop application, vendor integrations, or persistence. No provider-directory submission or ATS test is authorized by this document.
