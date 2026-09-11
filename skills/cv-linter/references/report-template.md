# CV Linter report template

Read this reference before presenting any CV Linter result. Adapt the template to the user's requested operation; omit optional or empty sections instead of filling them with boilerplate.

## Shared reporting rules

- Lead with the outcome, including an extraction failure or error-severity finding when present.
- Keep the tool's deterministic findings separate from host-AI assessment. Never change a finding's rule ID, severity, message, or source in the underlying result.
- Treat `findings` as non-spelling findings and `spelling_findings` as a separate resolution queue. Include all non-spelling errors and warnings. Informational findings may be grouped when every actionable observation and available human-readable location remains visible.
- Resolve spelling findings according to `SKILL.md`. Omit confirmed terminology from the human report and show only unresolved spelling findings in a separate section.
- Use only source locations and block IDs returned by the current result. Use block IDs and byte ranges internally; do not display them by default. Present a PDF page, Markdown line, DOCX element/table location, a short contextual label, `document-level`, or `location unavailable` as appropriate. Never infer missing precision.
- Qualify host assessment as `likely actionable`, `likely intentional`, or `uncertain`, with a short reason. These labels do not replace deterministic severity.
- Avoid reproducing raw CV content unless the user requested it or a short cited excerpt is necessary to explain a finding.
- Do not produce ATS scores, hiring predictions, candidate rankings, certifications, or vendor-compatibility claims.
- If extraction handed CV text to the host, retain the privacy distinction: parsing and linting were local, while host/model processing follows the host's data policies.

## Complete CV check

Use this shape for a general check or review after `lint_cv`, spelling resolution, `extract_cv_text`, and reading `cv-writing-guidance.md`:

```markdown
# CV Linter report — <filename>

<One-sentence outcome naming the most important actionable issue. Do not count closed spelling terminology as an issue.>

## Deterministic lint

| Severity | Observation | Where | Assessment |
|---|---|---|---|
| Warning | <faithful deterministic observation> | <human-readable returned location> | <likely actionable / likely intentional / uncertain — reason> |

## ATS-oriented risks

| Observation | Why it matters | Where | Suggested action |
|---|---|---|---|
| <grounded host assessment> | <calibrated ATS-oriented rationale> | <human-readable returned location> | <bounded advice> |

## CV best practices

| Observation | Why it matters | Where | Suggested action |
|---|---|---|---|
| <grounded host assessment> | <guidance, not a universal rule> | <human-readable returned location> | <source-preserving advice> |

## Spelling

| Term | Where | Suggested action |
|---|---|---|
| <unresolved term only> | <human-readable returned location> | <verify or correct; do not invent a correction> |

## Extraction limitations

- <Only warnings or limitations returned by the tool.>

## Suggested next actions

1. <Highest-value grounded action.>
```

Order non-spelling findings by error, warning, then informational severity. Omit empty sections, including Spelling when every term was closed. Keep ATS-oriented and best-practice assessments explicitly labeled as host interpretation. If the result is too large to present completely, disclose truncation and counts instead of silently omitting actionable findings.

For a deterministic-only lint request, use just the outcome, Deterministic lint, Extraction limitations, and Suggested next actions sections. Do not extract text or add ATS-oriented or best-practice conclusions.

Technical run identity—status, block count, schema, ruleset, extractor, input hash, rule IDs, internal block IDs, and normalized allow words—is available for debugging and reproducibility. Include it only when the user requests technical detail or the task is a comparison run.

## Extraction-only report

When the user asks only for extraction, report status, extractor identity, input hash, block count, and returned warnings or limitations. Summarize block kinds or source coverage when useful. Do not reproduce every extracted block unless the user asked to see the text.

## Semantic review or job comparison

Use this only after the user requested semantic work and the extraction handoff was authorized. Start with the deterministic lint summary above, then add only the applicable sections:

```markdown
## Requirement comparison

| Job requirement | Verdict | CV evidence | Rationale | Confidence |
|---|---|---|---|---|
| <discrete requirement> | `supported` / `partially_supported` / `not_evidenced` / `uncertain` | <human-readable returned location or `none`> | <short grounded explanation> | high / medium / low |

## Reviewable rewrite suggestions

### <purpose>

- Before: <exact or faithfully scoped current wording>
- After: <source-preserving suggestion>
- Evidence: <human-readable returned location grounded internally in a block ID>

## Open questions

- <Focused question whose answer would resolve missing or uncertain evidence.>
```

Retain the job description's meaning when splitting it into requirements. Say `not evidenced in the supplied CV`, never that the candidate lacks a skill. A rewrite must preserve confirmed names, employers, dates, credentials, numbers, skills, and responsibilities. Do not add metrics or implications that the cited blocks do not support, and do not edit the source CV without a separate explicit request and approval.

## Comparison runs

For plugin-versus-ordinary-agent evaluation, keep the user prompt and requested report sections the same. Record schema, ruleset, extractor, exact input hash, and operation-scoped options for the plugin run. Record unavailable metadata as unavailable in the ordinary-agent run rather than inventing equivalents. When rule configuration exists, also record its effective identity or hash.
