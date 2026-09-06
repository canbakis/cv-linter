# CV Linter: Agent Skill Architecture

**Date:** 6 September 2026
**Status:** Proposed contracts and package design; no executable, skill installation, MCP server or application code is implemented by this document. Examples are specifications, not working commands.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Agent and PWA UX](ui-design.md)

## 1. Decision, claims and invariants

Publish an Agent Skills-compatible core with a local executable as the primary execution path, an optional local stdio MCP wrapper, thin Hermes/Claude Code/Codex adapters, and a PWA using the same engine and report schemas. This is a design inference from the [Luna distribution memo](research/agent-skill-distribution-research.json), refined by the owner's direction. It is not a requirement imposed by a host or evidence of market demand.

The [claims policy](architecture-and-product-plan.md#1-product-decision-and-claims-policy) applies here: vendor/platform facts require scoped primary citations; our architecture is a design decision; proposed budgets and interoperability are unvalidated until tested. The [ATS memo](research/ats-vendor-research.json) and [judge memo](research/llm-judge-research.json) remain research inputs, not runtime instructions. Package contents must not turn speculative vendor behavior into hard universal rules.

Required invariants:

1. `lint`, `compare-to-job`, and `report` are deterministic, network-free operations. They never initialize, download, or call models. No `judge_enabled` switch or `lint --judge` shortcut exists in v1; reject such fields as invalid input.
2. `judge` is a first-class supported command from the initial product scope. Execution requires an explicit request and valid authorization over an immutable payload; local inference is preferred and remote inference is explicit opt-in.
3. A judge can add advisory records only. It cannot modify source bytes, extraction, deterministic findings, rubric arithmetic, scores, profile constraints or eligibility outcomes.
4. The engine accepts user-selected file capabilities or bounded bytes. Neither a current working directory nor an agent's broad access becomes CV Linter permission.
5. A local process, stdio transport or `SKILL.md` is not a privacy guarantee. Host-context disclosure and judge-provider disclosure are separate boundaries.
6. Every result records versions and evidence provenance. Every attempted judge execution produces a controller-owned audit outcome, including failure and abstention.

## 2. Package tree and release contract

The portable skill root is named `cv-linter`; build-time tooling produces identical core contents for every host wrapper. The following is a **proposed release tree**, not files to create in this planning task.

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
│   │   ├── cv-linter                # fixed launcher, no install-on-demand
│   │   └── cv-linter-mcp            # optional stdio launcher
│   ├── lib/
│   │   ├── core/                   # parser, normalizer, rules, report, validation
│   │   ├── controller/             # file capabilities, consent, audit, limits
│   │   └── inference/              # explicit local/remote transport adapters
│   ├── schemas/
│   │   ├── requests.schema.json
│   │   ├── document-evidence.schema.json
│   │   ├── lint-result.schema.json
│   │   ├── job-comparison.schema.json
│   │   ├── judge-output.schema.json
│   │   ├── consent-receipt.schema.json
│   │   ├── audit-event.schema.json
│   │   └── report-bundle.schema.json
│   ├── rules/                      # declarative definitions, no executable plugins
│   ├── profiles/                   # generic + seven vendor scopes with citations
│   ├── prompts/                    # versioned bounded judge templates
│   ├── rubrics/                    # task criteria and abstention policies
│   ├── references/
│   │   ├── commands.md
│   │   ├── claims-policy.md
│   │   ├── evidence-and-judging.md
│   │   └── vendor-sources.json
│   ├── assets/
│   │   ├── parser/                 # pinned local WASM/fonts/data where required
│   │   ├── report/                 # local renderer assets/templates
│   │   └── model-catalog.json      # artifact IDs/licenses/digests, not weights
│   └── fixtures/                   # small synthetic smoke fixtures, no real CVs
├── adapters/
│   ├── hermes/                     # native skill/hub metadata and setup guidance
│   ├── claude-code/                # optional plugin manifest + skill/MCP wrappers
│   └── codex/                      # native metadata and optional plugin wrapper
├── pwa/                            # separate build of shared core/report contracts
└── artifacts/                      # platform executables and optional model packs
```

The distributable launcher resolves only its verified package/runtime; it must not use a remote package runner, dynamically install dependencies, or let a CV choose a module. Optional platform/model artifacts are independently versioned and digest-bound to the release manifest. Model weights are not downloaded on skill discovery or first lint. Large/consented benchmark datasets stay outside the public skill; only licensed synthetic smoke fixtures ship.

Agent Skills defines YAML frontmatter and a Markdown body with optional scripts/references/assets. It does not standardize CV Linter's manifest, signature scheme, runtime dependencies or sandbox. Its experimental `allowed-tools` field has host-dependent support. [Agent Skills specification](https://agentskills.io/specification).

Illustrative portable frontmatter, followed by short imperative workflow instructions:

```yaml
---
name: cv-linter
description: Check a user-selected CV with deterministic local linting and evidence reports. Explicit advisory judging is supported as a separate consented operation; a normal check never invokes it.
compatibility: Requires a supported local CV Linter executable. Lint works offline; model setup and remote inference require separate explicit actions.
metadata:
  version: "0.1.0"
  contract-version: "1.0.0"
---
```

These version values are illustrative. Keep host-only invocation/permission keys out of portable frontmatter; generate and test them in adapters. The body tells the agent to select the fixed command, resolve a single authorized input, preserve returned findings, respect projection/consent boundaries, show abstention, and offer local evidence inspection. Do not embed parsing logic or judge prompts for free-form execution by the conversational host.

### Manifest and versioning

`manifest.json` must declare package ID/SemVer, source revision, build recipe/toolchain digest, supported OS/architecture/runtime matrix, entrypoints, schema/protocol versions, component versions and hashes, parser assets, license/SBOM references, dependency locks, optional model/runtime IDs, and capabilities required per operation. Declare `lint.network = none`, input-only reads and explicit output permissions. These declarations describe policy; the controller and OS/host configuration enforce it.

Independently pin parser, normalizer, rule pack, vendor profile, scoring rubric, schema, prompt, advisory rubric, model artifact/quantization, inference runtime and host adapter. Use SemVer for public contracts and components where applicable; also record immutable digests. A parser/rule/prompt change requires a component bump even when its interface stays compatible. Breaking schemas/command semantics require a major contract version; reject unsupported majors. Meaning changes require new profiles/rubrics and invalidate comparisons that assume the old meaning. Record explicit compatibility ranges; “latest” is not reproducible.

Avoid circular integrity definitions: the manifest inventories payload files but excludes itself, checksum inventory and signature. `checksums.sha256` covers the manifest and remaining payload files, excluding itself and its signature; sign that exact inventory. Authenticate the signature with a separately trusted publisher key. The launcher/verifier must itself be obtained through a trusted signed release/install path; a modified verifier cannot attest to its own honesty.

Produce deterministic archives with sorted paths, fixed metadata and a pinned build environment, and publish source/build provenance. Rebuilding identical artifacts is a **release target to verify**, not a current achievement. Verify integrity before reading documents; fail closed on unknown/changed executable, rule or prompt files. Updates are explicit and atomic, preserve the previous verified version for rollback, require review of new permissions, and never occur mid-analysis. Production adapters cannot rewrite the installed skill; custom development builds have a separate identity and disclose unverified status.

Uninstall removes package/model assets under their declared roots. It does not silently delete user reports/history; provide a separate explicit deletion action. Host configuration or registry installation is future user-authorized setup, not a side effect of lint.

## 3. Command and API contracts

All names below are proposed v1 interfaces. Commands accept bounded structured requests, reject unknown keys/options and avoid shell interpolation. The launcher uses fixed argument arrays. The in-process API mirrors these operations over byte/input handles; browser callers use File/Blob capabilities rather than native paths.

| Command / API | Inputs | Result and authority |
|---|---|---|
| `lint` / `lint(request)` | One `input` (selected path, bytes or stdin), pinned rules/profile, locale, declared operational limits | Immutable lint result, evidence store and capabilities; all checks deterministic |
| `compare-to-job` / `compareToJob(request)` | Verified lint result plus explicit job bytes/path; reviewable requirement inventory and vocabulary version | Deterministic requirement-to-span matrix; exact/curated-synonym matches labeled separately; no semantic inference |
| `judge prepare` / `prepareJudge(request)` | Verified lint/comparison handles, task/criteria, model registry ID, local/remote mode, redaction/scope/attempt budget | Immutable proposal and local preview handle; no model call; may return setup/scope/consent requirements |
| `judge run` / `runJudge(request)` | Prepared proposal ID and trusted consent receipt bound to it | Validated advisory result plus controller audit; never accepts raw ad-hoc prompts or replacement evidence |
| `report` / `renderReport(request)` | Existing versioned report bundle, format, explicitly selected projection/output; optional baseline report | Offline JSON/text/self-contained HTML export or deterministic revision diff; no inference, reparsing or source reread |

Explicit developer/maintenance commands may include `version --json`, `verify`, `models list`, user-triggered model setup and `mcp --stdio`; none analyze a CV. A no-argument invocation prints usage and does not search for a CV. No generic shell execution, URL-fetch or directory-ingest command exists.

Illustrative standalone sequence; paths must already be authorized and these files would be explicit user-created outputs:

```sh
cv-linter lint --input ./cv.pdf --format json --output ./reports/lint.json
cv-linter compare-to-job --lint ./reports/lint.json --job ./job.txt --output ./reports/comparison.json
cv-linter judge prepare --lint ./reports/lint.json --task clarity --mode local --model installed-model-id --output ./reports/proposal.json
cv-linter judge run --proposal ./reports/proposal.json --interactive --output ./reports/advice.json
cv-linter report --input ./reports/lint.json --advice ./reports/advice.json --format html --output ./reports/review.html
```

`--interactive` opens the trusted local disclosure/consent step, not an unconditional approval flag. Noninteractive runs use `--consent-receipt` referencing a previously issued, matching local receipt. Remote preparation additionally requires an explicitly configured provider ID; environment credentials alone do not opt in. Never put CV/JD text, provider secrets or bearer consent tokens in command arguments/history. Proposed user-facing syntax is adapted to each host; it is not a promise that hosts share slash-command grammar.

### Common request and result envelope

Requests declare `contract_version`, `operation`, `request_id`, one input capability (exactly one path/bytes/handle alternative), component selections, locale, limits and output policy. The controller resolves versions and checks permissions before processing. Judge runtime identity and endpoints come from an approved registry, not model-generated URLs or arbitrary executable paths. No input can increase the controller's installation/session policy.

Results include `schema_version`, `operation`, `status`, immutable `analysis` and a separate `execution` envelope. Analysis records SHA-256 input hashes, parser/normalizer/rules/profile/rubric versions, capabilities, applicable checks, `pass|partial|fail|unknown|not_applicable` outcomes, evidence IDs, severity, observation certainty, citations, unassessed scope and warnings. Execution records run ID, host/OS/runtime, duration and cancellation/errors. Canonical hashing excludes volatile execution fields and sorts stable rule/span IDs. Local evidence bundles can include private text; host projections are separately authorized outputs.

No aggregate is issued for empty/unreadable input or zero applicable weight. A failed parser/disabled applicable check is `unknown`; absence of an input capability can make a layout check `not_applicable`. Preserve the product plan's provisional scoring/coverage policy; judge records have no compatibility-score contribution.

Proposed CLI exit codes: `0` completed operation (including visible advisory abstention), `1` deterministic findings meet the explicitly configured fail threshold, `2` invalid/unsupported request or denied permission, `3` resource/parser/runtime failure, `4` consent/model setup required, `5` invalid judge output, `130` cancelled. The structured status is authoritative about what completed; code 0 is never an ATS-pass claim. JSON goes to stdout only when selected; diagnostics are content-free stderr. MCP always reserves stdout for protocol frames. Timeouts, output/token/array limits and cancellation apply to every entrypoint.

### Input and report identity

An analysis uses opened bytes and a hash, not a mutable filename. Stored reports are not trusted merely because they contain hashes. Judge preparation requires current controller-verified evidence or a verified retained bundle; unverifiable imports return `evidence_unverified` and require an explicit lint operation. Rendering an import labels its provenance and cannot execute embedded instructions. Revisions, changed rules or changed normalization cannot reuse prepared payloads or consent.

`report --baseline` compares deterministic results only under compatible rule/profile/capability versions, otherwise explains why a direct comparison is withheld. Source-text changes and potential losses are separate from resolved findings. Pairwise judge review, if offered, is restricted to two revisions of the same user-selected CV and consumes a separate explicit, bounded multi-run authorization.

## 4. Evidence, judge responsibilities and consent

Each native document source has a stable `source_id`, original content hash and parser version. Immutable extraction spans have `span_id`, raw extracted text, normalization mapping, provenance and a format-specific locator. Use zero-based Unicode code-point offsets, half-open `[start, end)`, with explicit conversion for JavaScript UTF-16 APIs. PDF locators carry one-based page and coordinate space; DOCX locators carry OOXML part/paragraph/run references, not invented page coordinates; text locators include line and offsets.

A judge payload is an immutable **view** of selected evidence. Its span IDs and offsets address the exact transmitted view, including redaction. A private mapping resolves surviving quotes back to native spans/locations. Redacted gaps, excluded sections, uncertain OCR and retrieval truncation are explicit. A quote cannot cross a removed gap or be treated as original evidence if it was user/model reconstruction. The model may select spans but may not invent coordinates, files or evidence stores.

Supported task IDs are `clarity`, `accomplishment_evidence`, `semantic_mapping`, `job_evidence`, and `revision_review`. Task rubrics enumerate allowed criterion IDs and what each label means. Allow source-preserving wording suggestions and evidence explanations; forbid applicant ranking, hiring decisions, unsupported qualifications/numbers and protected-attribute judgments. Dates/literal credentials and arithmetic remain deterministic evidence, not model determinations. An absent claim is not contradiction or proof of absence in the person's life.

The controller prepares at most a declared number of criteria/spans/tokens/attempts, with explicit retrieval coverage. If full context cannot fit, propose a narrower assessed scope and disclose it before consent. Never judge an incomplete scope as if it were the whole CV. Abstention covers missing context, extraction ambiguity, conflicting evidence, unsupported language, injection concerns, low confidence and resource exhaustion.

Consent binds proposal ID, input/report/payload hashes, task and criteria/order, model/provider/endpoint and any relay, artifact/runtime/prompt/rubric versions, redaction/scope, retention/training disclosure, destination, expiration, maximum attempts/tokens/cost and approving actor/channel. A trusted local controller/UI issues a signed or session-MACed single-use receipt after the user's action. Receipt issuance must distinguish a real user decision from agent-driven UI automation; it is not exposed as a callable model capability. The model cannot mint receipts; token possession is not sufficient without binding, signature and replay checks. Tokens are opaque local handles in host results, not reusable bearer secrets in chat. Batch receipts list exact immutable proposals and finite budgets; no blanket “all future CVs” approval.

Local request/confirmation and host permissions already covering the exact disclosed operation are sufficient; avoid redundant prompts. Remote mode requires prior provider opt-in and exact-payload approval. Changing a model, destination, source, prompt, redaction, rubric or selected content requires a new proposal/consent. The controller validates permission and freshness again immediately before execution and reserves/consumes the receipt atomically; duplicate MCP requests return existing operation state rather than sending twice.

## 5. Strict judge output schema

This complete proposed JSON Schema describes the **untrusted model response only**. Controller-generated consent, model identity, report hashes, transport errors and audit metadata are stored outside it. The response is a single JSON object, without Markdown or executable content. All object keys are required and closed; nullable fields are explicit. Constraints are the authoritative contract even if a chosen model runtime cannot enforce them while decoding; post-validation is mandatory.

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

Invalid JSON/evidence produces controller status `judge_invalid`, an audit event and no accepted advisory payload. Do not silently patch the response, coerce enums, accept partial invalid findings, or request repair. An explicitly authorized bounded retry is a new recorded attempt against the unchanged proposal and budget; changing context requires new consent. Schema validity is necessary but does not establish factual correctness or absence of bias.

## 6. MCP tool boundaries

MCP is optional packaging for typed operations; it is not itself a sandbox or permission grant. Use the reviewed 2025-11-25 protocol/tools baseline as a proposed target and negotiate/test actual client support. [Specification](https://modelcontextprotocol.io/specification/2025-11-25), [tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools). The distribution memo also cites [2026-07-28 prompts](https://modelcontextprotocol.io/specification/2026-07-28/server/prompts); retain that citation without assuming clients support it or requiring MCP prompts for v1.

| Tool | Boundary and output |
|---|---|
| `cv_linter_lint` | Selected input handle/path allowed by controller; typed lint result or safe local handle according to output-disclosure policy; no model/network |
| `cv_linter_compare_to_job` | Authorized CV/JD handles only; deterministic evidence matrix; no URL retrieval/model |
| `cv_linter_prepare_judge` | Known evidence handle, task and registry model ID; local proposal/preview handle; cannot issue user consent or send data |
| `cv_linter_judge` | Prepared proposal and receipt handles only; explicit bounded inference; validated advisory result/audit handle; no arbitrary prompt/endpoint |
| `cv_linter_report` | Existing result handles and explicit output/projection; bounded rendering/export or diff; no execution of report content |
| `cv_linter_get_evidence` | Exact source/span IDs from current session plus valid host-output disclosure receipt; bounded snippets only; no general filesystem access |
| `cv_linter_cancel` | Session-owned operation ID only; stops future work and records terminal state without claiming to retract remote data |

Use strict input/output schemas on both client and server. MCP annotations are hints, not enforcement: lint/comparison may be read-only about source documents, while report export, session state and judge audit have declared write effects. Judge is never marked side-effect-free simply because it does not edit the CV. Do not expose approval/receipt-minting, shell, file search, directory listing, package installation, arbitrary HTTP, application submission or candidate mutation tools.

The server is a child process over stdio with no public TCP/HTTP listener. Sanitize environment inheritance; permit only required runtime/configuration and explicitly selected credential access. Standard error logs carry no private content. File capabilities, opaque report handles, quotas, consent receipts and audit IDs are per-session; deny cross-session lookup. MCP roots are scope hints intersected with controller policy, not blanket authorization. Validate paths even if the host already approved the tool.

Return structured error codes, progress without content leakage, and cancellation/timeout outcomes. Do not retry an uncertain remote request automatically. Keep operation-ID/idempotency state to prevent duplicate provider sends; reconnection cannot recreate expired consent. Private resources must not be advertised or returned to a cloud host without projection consent. Disable prompts and model sampling in v1 so a client cannot substitute its own model for `judge`.

Remote MCP is outside initial distribution. Enabling it later would introduce an additional data recipient, authentication and transport design, and separate consent. A PWA cannot attach directly to stdio; use shared browser core or explicit offline report import first.

## 7. Filesystem, network and host adapters

The controller opens a finite allowlist of selected regular files. Canonicalize paths, reject traversal/symlink escapes and non-file devices, validate opened file identity against selection, and use stable open handles/bytes to address replacement races. A separately approved directory capability may allow output creation under that directory; it never enables recursive CV discovery. Prevent output symlink escapes, input overwrite and unrelated-file overwrite. Use private temporary/run directories with restrictive permissions and guaranteed best-effort cleanup on success/failure/cancel. No credentials, source files or unselected paths in diagnostics.

Parsing has no network permission and never follows document relationships, entities, links or embedded actions. Model execution has no file/tools authority; its adapter can read only selected approved model assets and submit only the immutable payload. Local inference must use verified non-forwarding configuration and, where supported, OS egress denial. A service on loopback is classified `forwarding_unknown` until established otherwise and cannot receive a “local only” badge. Remote endpoint allowlisting, TLS and redirects are controlled independently of model output.

| Adapter | Documented platform basis | Proposed thin behavior and validation |
|---|---|---|
| Hermes | Skills and MCP are documented host features. [Skills](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills), [MCP](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp) | Map native invocation to fixed executable/tools; isolated configuration; verify the installed package if the host can edit skills; test explicit/implicit selection and transcript exposure |
| Claude Code | Skills, plugins and host permissions are documented. [Skills](https://docs.anthropic.com/en/docs/claude-code/skills), [plugins](https://docs.anthropic.com/en/docs/claude-code/plugins), [permissions](https://docs.anthropic.com/en/docs/claude-code/permissions) | Optional plugin carries namespace and local MCP declaration; use command/tool allowlists without bypassing consent; test native namespace and approval behavior against declared versions |
| Codex | SKILL.md packages support explicit/implicit use; permissions are host-configured. [Skills](https://developers.openai.com/codex/skills), [permissions](https://developers.openai.com/codex/permissions) | Generate optional `agents/openai.yaml`/plugin metadata in the host wrapper; use supported native sandbox and approvals, preserve fixed commands and output policy; no assumption that installed skills are automatically safe |

Actual adapter versions, packaging syntax and interoperability remain release validation work. Host permission is an upper bound intersected with CV Linter policy, never a reason to grant more access. Do not request blanket shell/network permission, install startup hooks, or let a host auto-judge after lint. If the host lacks a usable permission/disclosure boundary, report the limitation and offer standalone local execution.

Cloud-host tool results are a separate egress channel. By default return only opaque local handles and generic operation availability, without CV-derived summaries/hashes/excerpts. The local preview can issue a one-use disclosure receipt for an exact report projection. The server enforces it for all content-bearing tools/resources, independently of judge consent. A model must not bypass this by opening the same file with unrelated tools; adapter guidance and tested host restrictions are required, and a fully compromised/permissive host remains outside the engine's control.

## 8. Audit, reproduction and release acceptance

Store controller-generated events separately from untrusted model JSON. Required audit fields are schema/event/run/request IDs; event type/time; prior-event hash; input/lint/payload/output hashes (nullable when unavailable); package/component versions; model/provider/artifact/quantization/runtime and execution location/forwarding disclosure; prompt/rubric/task/order/decoding settings; consent receipt/time/channel; redaction/selected-span IDs; allowed and actual attempts/tokens/cost when known; validation/abstention/errors; and retention/export policy. Redact credentials and private paths. Invalid output is hashed, not automatically logged raw.

States are prepared → consented → started → completed/abstained/invalid/error/cancelled, with declined/expired proposals recorded where the session exists. Reserve consent and append the start event before any send; fail closed if the required record cannot be created. If completion recording fails, preserve a recoverable incomplete outcome and do not advertise a successful auditable result. Session events are append-only while retained; export includes the receipt/chain. Optional durable storage is encrypted and explicitly chosen. Compute each event hash over canonical event fields excluding its own hash/signature. A hash chain is tamper-evident relative to a trusted checkpoint, not proof against whole-log replacement or deletion by the device owner.

Default session receipts omit raw content; explicit local/encrypted bundles may retain exact payload/output for reproducibility. Hashes alone cannot replay deleted evidence. A deterministic rerun compares canonical analysis bytes with identical inputs/options/components and supported environment; host run metadata is separate. Model reruns record all settings and repeated trial outcomes but may differ even at fixed seed. Remote aliases may not offer immutable weights; disclose reduced replayability.

Release acceptance follows the [benchmark plan](architecture-and-product-plan.md#10-evaluation-dataset-and-benchmark-strategy): parser/rule correctness, schema/span rejection, scoped vendor claims, package verification/rollback, filesystem races, no inference from lint, no unauthorized host/provider transmission, duplicate/replayed consent, offline installs, runtime/adapter/PWA parity and cancellation. Judge acceptance additionally measures evidence entailment, objective factual fixtures, abstention usefulness, identity/proxy/verbosity/model-family bias and position bias by A/B reversal, criterion/span permutation and repeated trials. Pairwise production review is editorial revision comparison only.

No empirical gate has passed in this documentation task. Publish actual counts, configurations, intervals and limitations after testing. A model/task that has not met its declared gates stays experimental; judge support remains a visible command with setup/evaluation status, never a silently invoked lint dependency.
