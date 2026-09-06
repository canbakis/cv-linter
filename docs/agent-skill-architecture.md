# CV Linter: Agent Skill Architecture

**Date:** 6 September 2026
**Status:** Proposed contracts and package design; no executable, skill installation, MCP server or application code is implemented by this document. Examples are specifications, not working commands.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Agent and PWA UX](ui-design.md)

## 1. Decision, claims and invariants

**MVP uses a bundled local executable plus an Agent Skill. MCP is optional and deferred beyond MVP.** Thin Hermes/Claude Code/Codex adapters invoke the same commands; the PWA shares browser-compatible core modules and report schemas. No MCP installation, SDK or server is needed to lint, compare, judge or export. The [MCP decision and use cases](architecture-and-product-plan.md#is-mcp-necessary) explain when a later adapter could be worthwhile.

This design revises the [distribution memo](research/agent-skill-distribution-research.json), whose recommendations are research inputs rather than requirements. The [claims policy](architecture-and-product-plan.md#1-product-decision-and-claims-policy) applies: platform/vendor facts need scoped sources, interoperability needs testing, and model advice is not a deterministic fact. The [ATS memo](research/ats-vendor-research.json) and [judge memo](research/llm-judge-research.json) remain unchanged.

Required invariants:

1. `lint`, `compare-to-job`, and `report` are deterministic, local operations with no network requests or model initialization/download. Reject `judge_enabled` and `lint --judge`; a normal check cannot invoke inference.
2. `judge` is a first-class supported operation, executed only after a user reviews provider/model, selected content, exact payload and **ZDR / non-ZDR / unknown** for a provider request, then confirms. An on-device model also requires preview and confirmation; it is labeled local, not ZDR. No automatic fallback.
3. Advice cannot change source bytes, extraction, deterministic findings, rubric arithmetic, scores, profile constraints or eligibility outcomes.
4. Only explicitly selected files/bytes and output destinations are accessible to the engine; host permissions do not authorize directory discovery.
5. Local execution describes the engine's location. Agent chat/tool results follow the host's policies; disclose this briefly in setup/help and return useful, compact findings without a per-result projection gate.
6. Results retain versions and evidence provenance. Judge runs keep a basic controller-owned record of the confirmed request and outcome. Cryptographic consent receipts, hash-chained audits and durable encrypted history are deferred, not MVP dependencies.

## 2. Package tree and release contract

The portable skill root is named `cv-linter`; build-time tooling produces identical core contents for every host wrapper. The following is the **proposed MVP release tree**, not files to create in this planning task. It contains no MCP launcher, dependency or host server declaration.

```text
release/
├── cv-linter/                       # portable Agent Skills root
│   ├── SKILL.md                     # metadata, routing, boundaries, workflow
│   ├── LICENSE
│   ├── PRIVACY.md
│   ├── SECURITY.md
│   ├── CHANGELOG.md
│   ├── manifest.json                # CV Linter release manifest, not host standard
│   ├── checksums.sha256             # payload inventory plus manifest digest
│   ├── checksums.sha256.sig         # signature verified against external trust key
│   ├── dependencies.lock.json
│   ├── sbom.spdx.json
│   ├── build-provenance.json
│   ├── scripts/
│   │   └── cv-linter                # fixed launcher, no install-on-demand
│   ├── lib/
│   │   ├── core/                   # parser, normalizer, rules, report, validation
│   │   ├── controller/             # selected files, confirmation, run state, limits
│   │   └── inference/              # explicit local/remote transport adapters
│   ├── schemas/
│   │   ├── requests.schema.json
│   │   ├── document-evidence.schema.json
│   │   ├── lint-result.schema.json
│   │   ├── job-comparison.schema.json
│   │   ├── judge-output.schema.json
│   │   ├── judge-request.schema.json # exact prepared payload and disclosure
│   │   ├── run-record.schema.json
│   │   └── report-bundle.schema.json
│   ├── rules/                      # declarative definitions, no executable plugins
│   ├── profiles/                   # generic + seven vendor scopes with citations
│   ├── prompts/                    # versioned bounded judge templates
│   ├── rubrics/                    # task criteria and abstention policies
│   ├── references/
│   │   ├── commands.md
│   │   ├── claims-policy.md
│   │   ├── evidence-and-judging.md
│   │   ├── vendor-sources.json
│   │   └── provider-policies.json  # public policy sources/scope/dates, no account data
│   ├── assets/
│   │   ├── parser/                 # pinned local WASM/fonts/data where required
│   │   ├── report/                 # local renderer assets/templates
│   │   └── model-catalog.json      # artifact IDs/licenses/digests, not weights
│   └── fixtures/                   # small synthetic smoke fixtures, no real CVs
├── adapters/
│   ├── hermes/                     # native skill/hub metadata and setup guidance
│   ├── claude-code/                # skill metadata + optional plugin wrapper
│   └── codex/                      # native metadata and optional plugin wrapper
├── pwa/                            # separate build of shared core/report contracts
└── artifacts/                      # platform executables and optional model packs
```

A future `cv-linter-mcp` package would be separately installed and versioned against this command/API contract; it is absent from MVP artifacts and adapter setup.

The packaged provider catalog records public policy eligibility, not a provider-wide ZDR assurance. Resolve the selected request's status using applicable account configuration at setup/run time; private account evidence stays outside the distributable skill. Missing evidence yields unknown.

The distributable launcher resolves only its verified package/runtime; it must not use a remote package runner, dynamically install dependencies, or let a CV choose a module. Optional platform/model artifacts are independently versioned and digest-bound to the release manifest. Model weights are not downloaded on skill discovery or first lint. Large/consented benchmark datasets stay outside the public skill; only licensed synthetic smoke fixtures ship.

Agent Skills defines YAML frontmatter and a Markdown body with optional scripts/references/assets. It does not standardize CV Linter's manifest, signature scheme, runtime dependencies or sandbox. Its experimental `allowed-tools` field has host-dependent support. [Agent Skills specification](https://agentskills.io/specification).

Illustrative portable frontmatter, followed by short imperative workflow instructions:

```yaml
---
name: cv-linter
description: Check a user-selected CV with deterministic local linting and evidence reports. Optional advisory judging shows the chosen model and exact payload for confirmation; a normal check never invokes it.
compatibility: Requires a supported local CV Linter executable. Lint works offline; model setup and remote inference require separate explicit actions.
metadata:
  version: "0.1.0"
  contract-version: "1.0.0"
---
```

These version values are illustrative. Keep host-only invocation/permission keys out of portable frontmatter; generate and test them in adapters. The body tells the agent to select the fixed command, resolve a single authorized input, preserve returned findings, show the judge disclosure and require confirmation, show abstention, and offer local evidence inspection. Do not embed parsing logic or judge prompts for free-form execution by the conversational host.

### Manifest and versioning

`manifest.json` must declare package ID/SemVer, source revision, build recipe/toolchain digest, supported OS/architecture/runtime matrix, entrypoints, schema/protocol versions, component versions and hashes, parser assets, license/SBOM references, dependency locks, optional model/runtime IDs, and capabilities required per operation. Declare `lint.network = none`, input-only reads and explicit output permissions. These declarations describe policy; the controller and OS/host configuration enforce it.

Independently pin parser, normalizer, rule pack, vendor profile, scoring rubric, schema, prompt, advisory rubric, model artifact/quantization, inference runtime and host adapter. Use SemVer for public contracts and components where applicable; also record immutable digests. A parser/rule/prompt change requires a component bump even when its interface stays compatible. Breaking schemas/command semantics require a major contract version; reject unsupported majors. Meaning changes require new profiles/rubrics and invalidate comparisons that assume the old meaning. Record explicit compatibility ranges; “latest” is not reproducible.

Avoid circular integrity definitions: the manifest inventories payload files but excludes itself, checksum inventory and signature. `checksums.sha256` covers the manifest and remaining payload files, excluding itself and its signature; sign that exact inventory. Authenticate the signature with a separately trusted publisher key. The launcher/verifier must itself be obtained through a trusted signed release/install path; a modified verifier cannot attest to its own honesty.

Produce deterministic archives with sorted paths, fixed metadata and a pinned build environment, and publish source/build provenance. Rebuilding identical artifacts is a **release target to verify**, not a current achievement. Verify integrity before reading documents; fail closed on unknown/changed executable, rule or prompt files. Updates are explicit and atomic, preserve the previous verified version for rollback, require review of new permissions, and never occur mid-analysis. Production adapters cannot rewrite the installed skill; custom development builds have a separate identity and disclose unverified status.

Uninstall removes package/model assets under their declared roots and preserves user-exported reports. History deletion behavior belongs to a future history feature. Host configuration or registry installation is future user-authorized setup, not a side effect of lint.

## 3. Command and API contracts

All names below are proposed MVP interfaces, not implemented commands. Accept bounded structured requests, reject unknown keys/options and use fixed argument arrays without shell interpolation. The in-process API mirrors the executable; browser callers use File/Blob capabilities rather than native paths. Schema validation belongs to this contract and does not require MCP.

| Command / API | Inputs | Result and authority |
|---|---|---|
| `lint` / `lint(request)` | One selected path/bytes/stdin input, pinned rules/profile, locale and operational limits | Immutable deterministic lint result and evidence |
| `compare-to-job` / `compareToJob(request)` | Verified lint result plus explicit JD bytes/path and vocabulary version | Deterministic requirement-to-span matrix; no implicit semantic inference |
| `judge` / interactive judge workflow | Verified report, task/criteria, selected model and provider if remote, scope and limits | Prepare locally, show disclosure and exact payload, require confirmation, then return validated advice and a run record |
| `judge prepare` / `prepareJudge(request)` | Same judge selections | Adapter-level preparation only: immutable proposal ID, exact payload, provider/model/location, scoped ZDR metadata and review state; no model call |
| `judge run` / `runJudge(request)` | Prepared proposal ID in an active controller session | Run only when that session records the user's confirmation of this exact proposal; otherwise `confirmation_required` |
| `report` / `renderReport(request)` | Existing report bundle, format, selected output content/destination and optional baseline | Offline JSON/text/self-contained HTML export or deterministic revision diff; no inference, reparsing or source reread |

`judge` is the ordinary user command. `prepare` and `run` expose the same internal stages for adapters, not mandatory extra steps or approval files for users. An interactive process can remain alive during review; MVP needs no always-running server. Developer/setup commands may include `version --json`, `verify`, `models list` and explicit model/provider setup. Model listings include execution location and applicable ZDR status; they do not send CV content. `mcp --stdio` and `cv-linter-mcp` are reserved for a later optional package, not available MVP commands. A no-argument invocation prints usage; there is no directory-ingest, general shell or URL-fetch operation.

Illustrative standalone sequence; paths and output files are explicitly selected:

```sh
cv-linter lint --input ./cv.pdf --format json --output ./reports/lint.json
cv-linter compare-to-job --lint ./reports/lint.json --job ./job.txt --output ./reports/comparison.json
cv-linter judge --lint ./reports/lint.json --task clarity --mode remote --provider configured-provider-id --model selected-model-id --interactive --output ./reports/advice.json
cv-linter report --input ./reports/lint.json --advice ./reports/advice.json --format html --output ./reports/review.html
```

The third command prepares the request, displays “Selected CV content will be sent to [provider] using [model],” **ZDR / non-ZDR / unknown**, the selected scope and exact payload, then waits for **Send and judge**. Choosing an installed model with `--mode local --model installed-model-id` instead displays **On this device · no provider transmission** and **Run local judge**; omit `--provider`. Local inference uses the same preview and confirmation.

`--interactive` enables review, not unconditional approval. Selecting a provider or supplying credentials never confirms a run. Noninteractive execution returns `confirmation_required` unless a supported trusted host UI has recorded the user's decision for the current proposal in the active controller session. No model-supplied `consent=true`, `--yes`, receipt file or blanket future-CV authorization is accepted in MVP. If a host cannot capture the user decision reliably, hand off to the interactive executable. Never put CV/JD text, secrets or authorization tokens in command arguments/history.

### Common request and result envelope

Requests declare `contract_version`, `operation`, `request_id`, an operation-specific input reference (exactly one path/bytes/handle alternative where applicable), component selections, locale, limits and output policy. The controller resolves versions and checks permissions. Model runtimes and provider endpoints come from configured registry entries, not document/model-supplied URLs or executables.

Results include `schema_version`, `operation`, `status`, immutable `analysis` and a separate `execution` envelope. Analysis records input hashes, parser/normalizer/rules/profile/rubric versions, capabilities, `pass|partial|fail|unknown|not_applicable` checks, evidence IDs, severity, observation certainty, citations and unassessed scope. Execution records run ID, runtime/location, duration, cancellation and errors. Canonical hashing excludes volatile execution fields and sorts stable rule/span IDs. Normal agent output is a compact findings summary with necessary evidence links/excerpts; complete evidence remains available in the local report.

No aggregate is issued for empty/unreadable input or zero applicable weight. A failed parser or disabled applicable check is `unknown`; absent format capability can make a layout check `not_applicable`. Preserve the product plan's provisional scoring/coverage policy; judge records have no compatibility-score contribution.

Proposed CLI exit codes: `0` completed (including visible advisory abstention), `1` deterministic findings meet a configured fail threshold, `2` invalid/unsupported request or denied permission, `3` resource/parser/runtime failure, `4` confirmation/model setup required, `5` invalid judge output, `130` cancelled. Structured status specifies what completed; code 0 is not an ATS-pass claim. JSON goes to stdout only when selected; diagnostics are content-free stderr. Timeouts, bounded output and cancellation apply to every entrypoint.

### Input and report identity

Analysis binds to opened bytes and a hash, not a mutable filename. Judge preparation needs current controller-verified evidence or a verified retained bundle. Unverifiable imports return `evidence_unverified` and require an explicit lint operation. Imported reports can be rendered with unverified provenance but cannot execute instructions or fetch sources. A changed revision or request invalidates judge confirmation.

`report --baseline` compares deterministic results only under compatible rule/profile/capability versions; otherwise explain why comparison is withheld. Potential source-content losses are separate from resolved findings. If editorial comparison of two revisions is offered through `judge`, both user-selected revisions and any multi-attempt budget must appear in its reviewed payload. It never ranks applicants.

## 4. Evidence, judge responsibilities and consent

Each native document source has a stable `source_id`, original content hash and parser version. Immutable extraction spans have `span_id`, raw extracted text, normalization mapping, provenance and a format-specific locator. Use zero-based Unicode code-point offsets, half-open `[start, end)`, with explicit conversion for JavaScript UTF-16 APIs. PDF locators carry one-based page and coordinate space; DOCX locators carry OOXML part/paragraph/run references, not invented page coordinates; text locators include line and offsets.

A judge payload is an immutable **view** of selected evidence. Its span IDs and offsets address the exact transmitted view, including redaction. A private mapping resolves surviving quotes back to native spans/locations. Redacted gaps, excluded sections, uncertain OCR and retrieval truncation are explicit. A quote cannot cross a removed gap or be treated as original evidence if it was user/model reconstruction. The model may select spans but may not invent coordinates, files or evidence stores.

Supported task IDs are `clarity`, `accomplishment_evidence`, `semantic_mapping`, `job_evidence`, and `revision_review`. Task rubrics enumerate allowed criterion IDs and what each label means. Allow source-preserving wording suggestions and evidence explanations; forbid applicant ranking, hiring decisions, unsupported qualifications/numbers and protected-attribute judgments. Dates/literal credentials and arithmetic remain deterministic evidence, not model determinations. An absent claim is not contradiction or proof of absence in the person's life.

The controller prepares at most a declared number of criteria/spans/tokens/attempts, with explicit retrieval coverage. If full context cannot fit, propose a narrower assessed scope and disclose it before consent. Never judge an incomplete scope as if it were the whole CV. Abstention covers missing context, extraction ambiguity, conflicting evidence, unsupported language, injection concerns, low confidence and resource exhaustion.

Preparation constructs the exact outbound request representation before confirmation: all CV/JD spans, instructions, rubric, report fields, attachments/metadata if any, model and generation settings. Show readable selected content and the complete request body, with no undisclosed additions after review; destination and any relay are disclosed separately. Credentials/authentication headers are excluded from preview and report. Scope changes or redactions rebuild the preview; a summary of fields alone is not an exact-payload preview.

Provider-policy metadata follows the [ZDR labeling policy](architecture-and-product-plan.md#zdr-is-provider-policy-metadata-not-a-guarantee): `zdr | non_zdr | unknown` internally, rendered **ZDR / non-ZDR / unknown**. Store source/contract reference, verified account/model/endpoint/feature/route scope, `last_verified_at`, review-due date and caveats. Missing, stale or conflicting evidence becomes unknown. Verified eligibility without account enablement is insufficient; a relay needs its own evidence. This metadata reports an applicable provider policy, not verified deletion, confidentiality or no-training guarantees. Non-ZDR and unknown requests remain selectable after disclosure and confirmation. On-device non-forwarding inference has no provider-policy ZDR label; display its local location instead.

The controller owns proposal and confirmation state. A trusted interactive UI or tested host user-input channel records a real user decision bound to the immutable payload, provider/model/route, ZDR disclosure, task/rubric and limits. The agent can prepare a proposal but cannot approve it through a tool argument or automated click. Recheck the binding and policy applicability immediately before execution; changed requests/disclosures require renewed review. Avoid duplicate confirmation for an unchanged, already confirmed operation, and atomically transition a run ID to started so duplicate calls cannot send twice.

MVP defaults to one attempt. Retries require an explicit decision after the failure/uncertainty is shown; no automatic provider switch or hidden repair conversation. Controller session expiry/restart clears unconsumed confirmation. General headless batches, signed/MACed consent receipts and a receipt-export workflow are deferred. These application checks do not claim to withstand a compromised host or OS.

## 5. Strict judge output schema

This complete proposed JSON Schema describes the **untrusted model response only**. Controller-generated confirmation, model identity, report hashes, transport errors and run metadata are stored outside it. The response is a single JSON object, without Markdown or executable content. All object keys are required and closed; nullable fields are explicit. Constraints are the authoritative contract even if a chosen model runtime cannot enforce them while decoding; post-validation is mandatory.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CV Linter advisory judge output v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "task", "rubric_id", "rubric_version", "judgments"],
  "properties": {
    "schema_version": { "const": "1.0.0" },
    "task": {
      "enum": ["clarity", "accomplishment_evidence", "semantic_mapping", "job_evidence", "revision_review"]
    },
    "rubric_id": { "type": "string", "minLength": 1, "maxLength": 128 },
    "rubric_version": { "type": "string", "minLength": 1, "maxLength": 64 },
    "judgments": {
      "type": "array",
      "minItems": 1,
      "maxItems": 32,
      "items": { "$ref": "#/$defs/judgment" }
    }
  },
  "$defs": {
    "evidence": {
      "type": "object",
      "additionalProperties": false,
      "required": ["source_id", "span_id", "start", "end", "quote"],
      "properties": {
        "source_id": { "type": "string", "minLength": 1, "maxLength": 128 },
        "span_id": { "type": "string", "minLength": 1, "maxLength": 128 },
        "start": { "type": "integer", "minimum": 0 },
        "end": { "type": "integer", "minimum": 1 },
        "quote": { "type": "string", "minLength": 1, "maxLength": 2000 }
      }
    },
    "judgment": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "criterion_id", "label", "confidence", "evidence_spans",
        "requirement_spans", "missing_evidence", "rationale",
        "suggestion", "abstain_reason"
      ],
      "properties": {
        "criterion_id": { "type": "string", "minLength": 1, "maxLength": 128 },
        "label": { "enum": ["supported", "contradicted", "insufficient_evidence", "abstain"] },
        "confidence": { "enum": ["low", "medium", "high"] },
        "evidence_spans": {
          "type": "array", "maxItems": 16,
          "items": { "$ref": "#/$defs/evidence" }
        },
        "requirement_spans": {
          "type": "array", "maxItems": 8,
          "items": { "$ref": "#/$defs/evidence" }
        },
        "missing_evidence": {
          "type": "array", "maxItems": 16,
          "items": { "type": "string", "minLength": 1, "maxLength": 500 }
        },
        "rationale": { "type": "string", "minLength": 1, "maxLength": 1500 },
        "suggestion": { "type": ["string", "null"], "minLength": 1, "maxLength": 1500 },
        "abstain_reason": {
          "enum": [null, "unreadable", "unsupported_language", "ambiguous_extraction",
            "missing_context", "conflicting_evidence", "suspected_injection",
            "low_confidence", "resource_limit"]
        }
      },
      "allOf": [
        {
          "if": { "properties": { "label": { "enum": ["supported", "contradicted"] } } },
          "then": { "properties": { "evidence_spans": { "minItems": 1 } } }
        },
        {
          "if": { "properties": { "label": { "enum": ["insufficient_evidence", "abstain"] } } },
          "then": { "properties": { "missing_evidence": { "minItems": 1 }, "suggestion": { "type": "null" } } }
        },
        {
          "if": { "properties": { "label": { "const": "abstain" } } },
          "then": { "properties": { "abstain_reason": { "type": "string" } } },
          "else": { "properties": { "abstain_reason": { "type": "null" } } }
        },
        {
          "if": { "properties": { "confidence": { "const": "low" } } },
          "then": { "properties": { "label": { "const": "abstain" } } }
        }
      ]
    }
  }
}
```

Semantic checks beyond JSON Schema are mandatory:

- Reject duplicate JSON keys, excessive bytes/depth, missing/duplicate/additional criteria, or task/rubric versions differing from the prepared request. One response per requested criterion is required.
- Resolve every source/span against the consented payload, validate `0 <= start < end <= span length`, and require exact code-point substring equality with `quote`. No external citations or guessed page numbers can substitute for evidence.
- CV evidence must reference an approved CV revision; `requirement_spans` must reference the supplied JD/requirement. `job_evidence` requires at least one exact requirement span per criterion, including absence/abstention cases. Other tasks require an empty requirement list unless the prepared rubric explicitly uses a supplied JD. Revision assertions must cite the relevant revision(s).
- `supported`/`contradicted` require positive evidence for the criterion's statement, not merely a keyword or absence. Semantic mappings require a span for the mapped phrase. Exact quote validity does not prove entailment; report advisory status and validate entailment on the benchmark. Suspicious unsupported additions are withheld for user review, not promoted as facts.
- `insufficient_evidence` is permitted only within a completely assessed declared scope and names what is missing. Retrieval gaps, uncertain extraction or omitted context require `abstain`. Low confidence always abstains. Do not turn abstention into pass or zero competence.
- Rationale is a short explanation tied to evidence, not hidden chain-of-thought. Suggestions may clarify supported claims but cannot add credentials, metrics, employers or eligibility. Judge output has no executable instruction or compatibility-score field.

Invalid JSON/evidence produces controller status `judge_invalid`, a failed run record and no accepted advisory payload. Do not silently patch the response, coerce enums, accept partial invalid findings, or request repair. An explicit retry is a new recorded attempt after the user reviews the failure; changing the payload, destination or disclosure requires a refreshed preview and confirmation. Schema validity is necessary but does not establish factual correctness or absence of bias.

## 6. Optional MCP adapter — deferred beyond MVP

MCP is an integration option, not the engine boundary or an MVP release requirement. The [use-case comparison](architecture-and-product-plan.md#is-mcp-necessary) recommends it only when a named host supports MCP but cannot run the bundled commands, or when measured cross-host discovery/evidence-query benefits justify the extra runtime and support work. JSON validation, model calls and confirmation already work through the executable and in-process API.

If that need is demonstrated, publish a separate local stdio adapter. Select and test an actual protocol/client version at that time; the memo's [2025-11-25 tool specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) is background, not an MVP version pin. The following is a **candidate later tool mapping**, not shipped v1 tools:

| Candidate tool | Existing controller operation and limit |
|---|---|
| `cv_linter_lint` | Selected input only; deterministic result and compact findings; no model/network |
| `cv_linter_compare_to_job` | Explicit CV/JD inputs; deterministic evidence matrix |
| `cv_linter_prepare_judge` | Prepared payload/disclosure; cannot confirm or send |
| `cv_linter_judge` | Active proposal with trusted user confirmation; same provider/model/ZDR preview and run record |
| `cv_linter_report` | Existing reports and selected output; render/export/diff only |
| `cv_linter_get_evidence` | Bounded spans from a session-owned report; no generic filesystem reader |
| `cv_linter_cancel` | Session-owned run ID; stops further work without claiming to retract data |

The optional adapter must preserve existing schemas and outcomes, use stdio without a public listener, isolate sessions, bound requests and validate selected paths. MCP annotations and roots are hints, not permission enforcement. Keep stdout for protocol frames and content-free diagnostics on stderr. Prevent duplicate provider sends after retries/reconnects; test progress, cancellation, failures and parity with the executable before release. A client approval is sufficient only if it presents the actual prepared request and captures the user's confirmation; a generic tool allowlist is not judge approval.

Do not expose shell/search/install, arbitrary HTTP, consent-minting, application submission or candidate mutation tools. No separate host-projection receipt system is added. Remote MCP, prompts and host model sampling remain separate future decisions, not prerequisites for local tools. The PWA continues to use shared browser code and explicit report import; a stdio adapter alone cannot connect it to a native runtime.

## 7. Filesystem, network and host adapters

The controller opens a finite allowlist of selected regular files. Canonicalize paths, reject traversal/symlink escapes and non-file devices, validate opened file identity against selection, and use stable open handles/bytes to address replacement races. A separately approved directory capability may allow output creation under that directory; it never enables recursive CV discovery. Prevent output symlink escapes, input overwrite and unrelated-file overwrite. Use private temporary/run directories with restrictive permissions and guaranteed best-effort cleanup on success/failure/cancel. No credentials, source files or unselected paths in diagnostics.

Parsing has no network permission and never follows document relationships, entities, links or embedded actions. Model execution has no file/tools authority; its adapter can read only selected approved model assets and submit only the immutable payload. Local inference must use verified non-forwarding configuration and, where supported, OS egress denial. A service on loopback is classified `forwarding_unknown` until established otherwise and cannot receive a “local only” badge. Remote endpoint allowlisting, TLS and redirects are controlled independently of model output.

| Adapter | Documented platform basis | Proposed thin behavior and validation |
|---|---|---|
| Hermes | Skills and MCP are documented host features. [Skills](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills), [MCP](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp) | Map native invocation to the fixed executable; isolated configuration; verify the installed package if the host can edit skills; test explicit/implicit selection and transcript exposure |
| Claude Code | Skills, plugins and host permissions are documented. [Skills](https://docs.anthropic.com/en/docs/claude-code/skills), [plugins](https://docs.anthropic.com/en/docs/claude-code/plugins), [permissions](https://docs.anthropic.com/en/docs/claude-code/permissions) | Optional plugin carries namespace and skill/executable metadata, with no MVP MCP declaration; use fixed-command allowlists and test native confirmation behavior against declared versions |
| Codex | SKILL.md packages support explicit/implicit use; permissions are host-configured. [Skills](https://developers.openai.com/codex/skills), [permissions](https://developers.openai.com/codex/permissions) | Generate optional `agents/openai.yaml`/plugin metadata in the host wrapper; use supported native sandbox and approvals, preserve fixed commands and output policy; no assumption that installed skills are automatically safe |

Actual adapter versions, packaging syntax and interoperability remain release validation work. Host permission is an upper bound intersected with CV Linter policy, never a reason to grant more access. Do not request blanket shell/network permission, install startup hooks, or let a host auto-judge after lint. If the host cannot run the fixed executable or capture judge confirmation, explain that specific limitation and offer the standalone executable/PWA; do not require MCP setup in MVP.

Agent results may enter a remote host's conversation. Include a short setup/help note explaining that the host processes and retains chat/tool results under its own policies. Return compact deterministic findings and necessary evidence rather than a full CV or raw payload; normal results need no separate projection receipt or privacy dialog. Respect any stricter host/user output setting. Offer standalone CLI/PWA inspection when the user wants to keep findings out of chat. Never claim that a local command makes a remote host's entire workflow local, and label execution in a remote agent runtime accurately.

## 8. Run records, reproduction and release acceptance

Keep controller-generated run metadata separate from untrusted model JSON. Record run/request ID and timestamps; input/report/payload identity; parser/rules/profile/schema and prompt/rubric versions; task and selected span IDs; model/provider/runtime and destination route; reviewed ZDR status/evidence/date; confirmation time/channel; configured limits and actual usage/cost when known; validation, abstention, cancellation and errors. Credentials and private paths do not belong in diagnostics. Raw payloads and replies remain in session for review as needed and are saved only through an explicit export; rejected replies are not routine logs.

Use ordinary session state: prepared → confirmed → started → completed/abstained/invalid/error/cancelled, with declined/expired requests handled without sending. Check confirmation and atomically reserve the run before inference. Retain the outcome in session and include basic Run details in explicit report exports. If recording fails after a request, show that the outcome is incomplete and whether a send may have occurred; never retry automatically. A cryptographic chain, signed consent receipt or durable audit database is not required for MVP.

Default storage is session/operation-scoped, plus user-selected report files with restrictive permissions and best-effort temporary-file cleanup. A simple Clear session removes current app state and stops work; it cannot delete prior exports or host/provider copies. Encrypted persistent history, retention management and replay bundles are later features to evaluate on demand. Use established encryption/key-storage facilities if those features are added.

For deterministic reproduction, compare canonical analysis bytes with identical inputs/options/components in a supported environment; exclude run timestamps and other volatile metadata. Record model settings to explain changes, but do not promise identical model output or immutable remote aliases. Hashes cannot reconstruct deleted evidence or prove provider deletion.

Release acceptance follows the [benchmark plan](architecture-and-product-plan.md#10-evaluation-dataset-and-benchmark-strategy): parser/rule correctness, schema/span rejection, scoped vendor claims, package verification/rollback, selected-file limits, no model/network from deterministic commands, actual payload-preview equality, confirmation enforcement, ZDR metadata applicability, duplicate-send prevention, cancellation and executable/adapter/PWA parity. Test host results for compact output and accurate processing-location labels, without requiring a separate projection approval. MCP conformance becomes a gate only for a future MCP release.

Judge acceptance additionally measures evidence entailment, objective factual fixtures, abstention usefulness, identity/proxy/verbosity/model-family bias and position bias through order reversal, permutations and repeated trials. Pairwise production review remains editorial revision comparison only. No empirical gate has passed in this documentation task; publish actual counts, configurations, intervals and limitations after testing. Unevaluated model/tasks stay experimental, with judge support visible and never an implicit lint dependency.
