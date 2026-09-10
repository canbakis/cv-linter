# CV Linter: Vendor Evidence and Authorized Testing Plan

**Date:** 10 September 2026
**Status:** Documentation research and proposed future testing; no vendor account access, fixture submissions, benchmarks, or outreach performed.
**Main plan:** [Product and architecture](../architecture-and-product-plan.md)
**Inherited research:** [ATS vendor memo](ats-vendor-research.json), retained as background input.

## 1. Recommendation and evidence hierarchy

**Keep this as a separate research document and make its evidence rules part of the main architecture.** Start the next research pass with **Teamtailor, Greenhouse, Lever, and Workable**. Keep the other inherited profiles as background research; their inclusion is not a promise of tested support. The MVP ships no vendor profiles, ATS integration, vendor uploads, or Teamtailor-compatible claim. Any future profile remains **documentation-based guidance** until authorized experiments produce reviewed evidence for a particular workflow.

This changes claim provenance, profile governance, evaluation, and report wording. It does not require a vendor account for normal linting or an MVP network integration with an ATS. Access research can proceed alongside local parser research. A sandbox invitation alone does not validate a profile.

The following is a **design decision**, not a ranking of vendors:

| Evidence category | Authority and permitted use | Boundary |
|---|---|---|
| Official product, support, API, and access documentation | Primary authority for what the vendor documents about a named feature, workflow, edition, or access route | A documented capability is not a measured extraction success rate or permission for our study |
| Authorized vendor measurements | Empirical evidence of behavior in the tested tenant, configuration, workflow, dates, and fixture population | Does not establish behavior in another employer's tenant, a universal parser score, or a hiring outcome |
| CV Linter parser compatibility checks | Direct evidence of our parser's extraction, reading order, field associations, and rule behavior against independent ground truth | Does not establish how Teamtailor, Greenhouse, Lever, or Workable reads the same bytes |
| Research papers and reproducible third-party studies | Evidence within their disclosed methods, data, and evaluated systems | Do not inherit a vendor's current identity, configuration, permissions, or results without checking |
| Candidate-facing best-practice blogs, SEO articles, anecdotes, and competitor scores | Hypothesis sources for fixture design and questions to investigate | No authority for universal ATS requirements, penalties, rejection claims, or scoring weights; vendor-hosted career advice has the same limitation |

This is a hierarchy of **authority for a particular claim**, not one numerical confidence ladder. Local measurements cannot be promoted into vendor evidence. Documentation and authorized observations may disagree; preserve both, investigate scope/version differences, and mark the disputed claim rather than silently replacing it.

For example, a blog's warning about columns can motivate a paired fixture. Our parser losing a field proves a local defect on that fixture. A vendor support page can independently document a layout risk. Only an authorized vendor experiment establishes the vendor's observed result on that fixture. The consequence linking those records remains an explicitly labeled inference.

## 2. Parser research separate from ATS testing

The [Teamtailor/parsekit-bin](https://github.com/Teamtailor/parsekit-bin) repository is evidence of a published Teamtailor build, not production ATS compatibility. It is forked from [scientist-labs/parsekit](https://github.com/scientist-labs/parsekit), not [excoffierleonard/parser](https://github.com/excoffierleonard/parser). Its Teamtailor-specific diff appears to concern packaging, build, and release changes rather than ATS-specific parser or scoring logic. `parsekit-bin` is a Ruby gem with a Rust extension, and its MuPDF dependency raises AGPL/commercial licensing questions.

Record a **time-boxed extraction bake-off** between excoffierleonard/parser and upstream ParseKit before selecting a parser. Use representative Swedish CV fixtures covering reading order, single/multi-column layouts, headings, Unicode, tables, lists, and useful source-structure preservation. Evaluate reading order, columns, headings, Unicode, tables, source-structure retention, dependencies, binary size, runtime/resource behavior, and library/dependency/data licensing. This is a local extraction study only; it must not be described as an ATS compatibility test. Keep versions, fixture IDs, results, limitations, and license decisions with the spike record.

## 3. Research snapshot and source status

**Research observation:** The review on 2026-09-08 used public official support, API, partner, and selected terms pages. It did not enumerate private endpoints, inspect an employer tenant, or review a negotiated customer/partner agreement. “Not located” below means not located in this review, not proof that a service does not exist. Public API documentation and public job listings are not anonymous resume-parser services.

**Verification convention:** Facts attributed to readable official text below are `verified_documentation`, with `last_verified_at = 2026-09-08` for that source review only. Explicitly identified retrieval failures remain `pending_source_recheck`, with `last_verified_at = null` if no prior verification date is recorded and `last_review_attempt_at = 2026-09-08`. Relative page labels such as “updated over a week ago” are not converted into invented publication dates. None of these dates records a parser test.

### Teamtailor

| Topic | Vendor fact or research gap | Official source |
|---|---|---|
| Parser and fields | Co-pilot documents personal information including profile image, work/education history, and skills/traits. Matching competencies depend on the account's existing skills/traits. It documents hidden-instruction detection and warnings that do not change the candidate's score. Co-pilot activation and settings matter. | [Co-pilot: Resume parser](https://support.teamtailor.com/en/articles/16127956-co-pilot-resume-parser) |
| File formats | Candidate screening lists resume-text inputs from DOCX, PPTX, PDF, Pages, TXT, and RTF. This is a screening input list; a complete upload/Co-pilot-parser format and size contract was not established. | [Co-pilot: Candidate screening](https://support.teamtailor.com/en/articles/10209597-co-pilot-candidate-screening) |
| Search/export | Candidate search supports field-specific `resume_text` queries, Boolean/phrase search, and a CSV export workflow. This does not establish that every parser field is exportable. | [Filter, segment, and export candidates](https://support.teamtailor.com/en/articles/6121249-filter-segment-and-export-candidates) |
| Screening | Employer-configured criteria use resume text and other candidate/application data, support unknown evaluations and manual overrides, and can drive configured stage movement. | [Co-pilot: Candidate screening](https://support.teamtailor.com/en/articles/10209597-co-pilot-candidate-screening) |

**Design inference:** Model base resume parsing, Co-pilot enrichment, search, and screening as different scopes. Detecting invisible text locally does not establish manipulation, predict a Teamtailor warning, or justify a rejection penalty. Profile-image extraction is not a recommendation to include a photo. Confirm the actual parser and field provenance before treating a candidate profile as raw parser output.

**Access fact:** The API uses an existing Teamtailor account and a key generated by a Company Admin. Key creator role and key scope are distinct: the documentation describes Public, Internal, and Admin scopes; candidate data requires the relevant Admin scope. “Open API” does not mean anonymous candidate parsing. [Use our Teamtailor API](https://support.teamtailor.com/en/articles/5963369-use-our-teamtailor-api).

**Access fact:** The integration support guide says accepted tech partners receive a sandbox and describes testing with the platform's full functionality. [Integrate your HR/recruitment tool](https://support.teamtailor.com/en/articles/5477200-integrate-your-hr-recruitment-tool-with-teamtailor). The linked [Partner Hub guide](https://teamtailor.notion.site/Integrate-with-Teamtailor-3919690568754aaaa7ffd681afe955f2?pvs=4) failed to fetch in this review; the readable support guide independently supports the sandbox fact. The [API reference](https://docs.teamtailor.com/) exposed no readable text here, so endpoint-level parsing behavior remains unverified.

**Research gap:** No public anonymous Teamtailor parser endpoint was located. Co-pilot entitlement in our prospective sandbox, eligible research use, machine-readable fields, trial availability for this purpose, automation, and publication permission all require confirmation. “Full functionality” does not establish any of those grants to CV Linter.

### Greenhouse

| Topic | Vendor fact | Official source |
|---|---|---|
| Upload formats | Candidate uploads accept DOC/DOCX, PDF, RTF, and TXT, with a documented 100 MB upload allowance. | [Supported candidate uploads](https://support.greenhouse.io/hc/en-us/articles/360052218132-Supported-formats-for-resumes-cover-letters-and-other-candidate-uploads) |
| Parsing | Parsing has a separate 2.5 MB limit; documented risks include images, columns, tables, headers/footers, text boxes, split letters, and obvious fake data. Failed parsing can leave the file attached and require manual field entry. | [Unsuccessful resume parse](https://support.greenhouse.io/hc/en-us/articles/200989175-Unsuccessful-resume-parse) |
| Search | The candidate search workflow documents full-text resume keyword search. | [Search resumes for keywords](https://support.greenhouse.io/hc/en-us/articles/115004600186-Search-resumes-for-keywords) |
| Screening | Configured application-question responses can trigger auto-rejection; this is a distinct application workflow. | [Auto-reject](https://support.greenhouse.io/hc/en-us/articles/360000653472-Auto-reject) |

**Design inference:** Keep upload acceptance, parser output, search visibility, and screening outcomes separate. A 3 MB PDF fitting the documented upload allowance does not fit the parsing allowance. Use plausible fictional personas, with synthetic status recorded in study metadata, to reduce placeholder-related confounding; never change a real candidate's history to satisfy a parser.

**Access facts:** Greenhouse's [integration partner program](https://www.greenhouse.com/integration-partner) describes application/qualification followed by sandbox access. The [Assessment API](https://docs.greenhouse.io/assessment.html) also documents a partnership-team sandbox route, specifically for assessment integrations. The [customer sandbox guide](https://support.greenhouse.io/hc/en-us/articles/17053185557787-Use-a-sandbox) documents a Pro-tier test environment, mock candidates/jobs, and API testing. The [Harvest API](https://docs.greenhouse.io/harvest.html) provides authenticated account-data operations and endpoint-specific permissions. These are developer/customer testing paths, not a public resume-parser benchmark service.

**Research gap:** No anonymous parser benchmark endpoint or generally available self-service research trial was established. Confirm which approved ingest workflow actually invokes parsing and which API/export fields expose its output. Assessment integration access does not imply parser-benchmark eligibility.

### Lever

| Topic | Vendor fact or inherited claim | Official source and status |
|---|---|---|
| Parser formats | **Memo-attributed, pending source recheck:** Word/DOCX, PDF, RTF, WordPerfect, HTML/MS Office HTML, and ODF parsing; JPG/PNG not parsed. | [Understanding resume parsing](https://help.lever.co/hc/en-us/articles/20087345054749-Understanding-resume-parsing) returned an authorization error; retain the memo and citation |
| API parsing and fields | **Verified documentation:** authenticated opportunity creation supports a resume with `parse=true`; explicit fields take precedence over parsed values. API file support includes JPG/PNG but image files are stored without being parsed. Resume metadata can contain work/education records with mixed possible origins. | [API: supported file formats](https://hire.lever.co/developer/documentation#supported-file-formats), [create an opportunity](https://hire.lever.co/developer/documentation#create-an-opportunity), [resumes](https://hire.lever.co/developer/documentation#resumes) |
| Search | **Memo-attributed, pending source recheck:** resume-only search differs from broader search of parseable attachments, notes, and feedback. | [Candidate search](https://help.lever.co/hc/en-us/articles/20087317030685-Searching-the-Database-for-Candidates), [advanced search](https://help.lever.co/hc/en-us/articles/20087212721309-Using-advanced-search-and-rediscovery); candidate-search retrieval redirected to a page with a rendering error |
| Screening | **Research gap:** the help center lists Talent Fit, but its article exposed a rendering error. No current screening/ranking claim is verified from that page. API archive-state operations alone do not establish automatic screening logic. | [Talent Fit in Lever](https://help.lever.co/s/article/Talent-Fit-in-Lever), [API workflow reference](https://hire.lever.co/developer/documentation#update-archived-state) |

**Design inference:** The accessible API independently supports the image attachment/parsing distinction without validating the inherited UI format matrix. A successful create response is not field accuracy. Supplied fields, existing contacts, and deduplication must be controlled in a future experiment so they cannot masquerade as extraction.

**Access fact:** Approved integration partners receive a sandbox, with sandbox OAuth registration and a separate API base URL. [Partner product integration process](https://hire.lever.co/developer/partner). The [API reference](https://hire.lever.co/developer/documentation#authentication) documents authenticated account access. No anonymous parser endpoint or self-service trial for this research was established.

**Permission fact:** Lever's published terms restrict access for benchmarking purposes, separately addressing direct competitors' access. [Terms of Service, introductory restrictions](https://www.lever.co/legal/terms-of-service). **Decision:** treat the proposed benchmark as restricted under those published terms unless Lever provides an express applicable agreement permitting it. A partner sandbox invitation by itself is insufficient; publication requires its own permission.

### Workable

| Topic | Vendor fact | Official source |
|---|---|---|
| Parser and fields | Recruiter upload documentation explicitly includes image-based CVs and CVs in any language; it describes contact/name, social links, and profile-picture extraction, potentially supplemented by sourcing tools. The upload workflow lists DOC/DOCX, PDF, RTF, and ODT. | [Individual and bulk resume uploads](https://help.workable.com/hc/en-us/articles/115012661408-Uploading-candidate-resumes-CVs-Individual-and-bulk-options) |
| Application file formats | Resume uploads allow PDF, DOC/DOCX, RTF, HTML, and ODT up to 5 MB; custom-question attachments have a separate 20 MB allowance and broader formats. | [Application form file types](https://help.workable.com/hc/en-us/articles/115012238108-What-types-of-files-can-be-uploaded-on-the-application-form) |
| Search | Search covers resume and other profile sections, supports Boolean and phrase queries, and applies fuzzy matching by default. | [Advanced/Boolean candidate search](https://help.workable.com/hc/en-us/articles/6058530293015-How-do-I-run-an-advanced-boolean-search-for-candidates) |
| Screening | Configured Yes/No application questions can disqualify candidates on a No response. | [Auto-disqualify with application questions](https://help.workable.com/hc/en-us/articles/115012238688-Auto-disqualify-candidates-using-application-form-questions) |

**Design inference:** Do not label every image-based CV an external parsing failure. CV Linter's initial lack of OCR remains a local limitation, even when Workable documents image parsing. Neither image-based document support nor general attachments establish JPG/PNG acceptance in the resume field. “Any language” is a vendor capability claim, not measured multilingual accuracy or a reason to extend our MVP's declared semantic coverage.

**Access facts:** The [API getting-started guide](https://workable.readme.io/reference/generate-an-access-token) describes an account access token with selected scopes. [Candidate creation](https://workable.readme.io/reference/job-candidates-create) accepts supplied fields and a resume attachment; the reviewed page does not establish a standalone parse-and-return-fields service. The [15-day trial guide](https://help.workable.com/hc/en-us/articles/1500004328522-The-Workable-free-trial-guide) explicitly suggests an internal job with a fake candidate. The [partner program](https://www.workable.com/partnership-program) offers sandbox credentials after approval. [Technology partnership options](https://www.workable.com/partnership-program/technology-partners) distinguish access routes and rate limits; confirm the route applicable to this study.

**Research gap:** No public anonymous parser endpoint was located. Trial API entitlement, parser equivalence between UI and API, synthetic batch automation, and permission to publish this study remain unconfirmed. A trial guide permitting a fake candidate does not establish permission for an automated benchmark.

## 4. Access and permission register

**Current CV Linter state:** No access acquired or outreach sent; vendor benchmarks are `not_run` for all four vendors. These are documentation findings about possible access routes, not credentials or grants held by us.

| Vendor | Public anonymous parser | Authenticated API/customer account | Trial | Partner sandbox / test tenant |
|---|---|---|---|---|
| Teamtailor | Not located | Account + Company Admin-created key; parsing trigger/output unconfirmed | Not established for research | Approved-partner sandbox documented; Co-pilot entitlement to confirm |
| Greenhouse | Not located | Harvest/account permissions; actual parser invocation to confirm | Not established for research | Integration-partner route; Pro customer sandbox with mock data/API testing |
| Lever | Not located | Account API/OAuth; documented opportunity parsing | Not established for research | Approved-partner sandbox and sandbox OAuth; benchmarking restriction still applies |
| Workable | Not located | Scoped account token; candidate-create API is not proof of parsing | 15-day trial; internal fake-candidate example documented | Approved-partner sandbox; trial/API features and route to confirm |

Each row inherits the access citations in its vendor section. No one should interpret “not located” as an invitation to probe hidden endpoints or use live job applications.

| Vendor | Synthetic testing evidence | Automated benchmark permission for CV Linter | Publication permission for CV Linter |
|---|---|---|---|
| Teamtailor | General partner integration testing documented; synthetic parser study scope unconfirmed | `unknown`; request fixtures, mechanism, volume, rates, feature access, and applicable terms | `unknown`; request explicit scoped publication grant |
| Greenhouse | Mock candidates and API testing documented for customer sandboxes | `unknown`; establish study-specific rights and resolve applicable use restrictions | `unknown`; sandbox access is not a publication grant |
| Lever | Partner development/testing documented; fixture-study scope unconfirmed | `restricted`; published benchmarking restriction requires an express agreement permitting this study | `unknown`; request explicit permission alongside the benchmarking agreement |
| Workable | Internal-job fake-candidate trial use documented | `unknown`; distinguish the documented trial example from automated fixture batches | `unknown`; request explicit scoped publication grant |

**Additional terms evidence:** Greenhouse's published MSA limits use, including competitive development, in section 4(c). Workable's published terms cover permitted purposes, service interference, and usage/tier limits in section 4, with confidentiality in section 13. These texts establish constraints to resolve, not a determination that CV Linter is a competitor or that a particular research agreement will be refused. [Greenhouse MSA](https://www.greenhouse.com/master-subscription-agreement), [Workable terms](https://www.workable.com/terms).

Teamtailor's applicable research/partner terms were not established; request them. Record the actual agreement, authorized grantor, tenant, features, fixture types, allowed actions, validity dates, limits, publication scope, and any exceptions before using a vendor environment. Public terms are research inputs; a negotiated agreement may differ. Unknown does not mean prohibited, and it does not authorize execution. Preserve testing and publication as independent decisions: a private permitted test may still be unpublishable.

## 5. Architecture and rule metadata

**Design decision:** Keep three independent records joined by stable IDs:

1. A versioned **source/claim registry** for official facts, research hypotheses, and design inferences. Capture source title, section, URL, source type, scope, review status/date, and conflicting or superseded claims.
2. A **permission register and vendor experiment archive** for actual access grants, dated methods, fixture hashes, raw observable results, and publication status. Keep credentials and non-public agreements outside distributed rule packs.
3. **Local benchmark records** for CV Linter's own parser/rule versions, inputs, observations, gold annotations, and measurements. These are research artifacts, not an MVP persistence feature. Reusing fixture IDs enables comparison; it does not turn local results into vendor results.

Future rules must resolve the following metadata per claim, even if normalized into referenced records instead of duplicated in each rule:

| Field | Proposed meaning |
|---|---|
| `claim_kind` | `vendor_fact`, `research_finding`, `design_inference`, `hypothesis`, `local_parser_check`, or `authorized_vendor_observation`. A local detector, cited vendor premise, and inferred consequence are separate linked claims. |
| `scope` | Vendor or `cv_linter`; product/edition, workflow/API path, feature flags, configuration prerequisites, tenant reference where applicable, formats, language, component/profile version, and assessed stage: acceptance, parsing, fields, search, screening, or ranking. Unknown dimensions stay explicit. |
| `source_url` | Primary URL for an externally sourced claim, with additional source IDs/title/section; nullable for a local claim, which instead requires an internal evidence/method reference. Never invent a vendor source for our observation. |
| `verification_status` | `unverified`, `pending_source_recheck`, `verified_documentation`, `verified_observation`, `conflicting`, or `superseded`. Document review and successful experiment execution are different states. |
| `testing_permission` | Structured record with independent synthetic-upload, automation, and publication states: `not_applicable`, `unknown`, `requested`, `permitted`, `restricted`, `denied`, `expired`, or `revoked`; link supporting grant/terms, grantor, tenant, limits, dates, and allowed scope. Use `requested` only after a request is actually sent. |
| `last_verified_at` | Nullable date/time when this claim's source or experiment evidence was last substantively checked. Keep publication/update time, last review attempt, and experiment time separately. |
| `benchmark_status` | Separate `local` and `vendor` entries: `not_applicable`, `not_run`, `planned`, `pilot`, `evaluated`, or `needs_retest`, with run IDs and covered scope. Neither access approval nor source review is an evaluated benchmark. |
| `evidence_level` | `hypothesis`, `published_research`, `official_documentation`, `local_measurement`, or `authorized_vendor_measurement`. This describes evidence origin, not an ATS score or a transferable strength rating. |

Retain rule ID/version, applicability, severity basis, abstention, unit semantics, corrections, and evidence references from the main plan. Unknown MB/KB encoding/unit semantics must not become exact byte thresholds. `local_measurement` and `authorized_vendor_measurement` require actual recorded results; none exists in this revision. A new local rule can initially cite documentation or a hypothesis without pretending it was measured.

**Illustrative metadata, not a benchmark result:** The Teamtailor Co-pilot hidden-instruction documentation is a `vendor_fact`, scoped to Co-pilot/settings; its source is the parser support page, `verification_status = verified_documentation`, `last_verified_at = 2026-09-08`, and `evidence_level = official_documentation`. Its study permissions are unknown, vendor benchmark is `not_run`, and a future local invisible-text detector would have its own rule and evidence. Do not mark the documented detection mechanism empirically verified.

The MVP executable has no vendor-profile surface and never selects credentials, fetches live vendor sources, or uploads a CV because of vendor research. A later research-only harness may perform approved vendor operations outside `lint`, `extract-text`, and the two stdio MCP tools; it is not a required MVP component. Import reviewed findings only through a separately approved future profile version, with explicit research provenance. Host-AI output cannot change these records or grant testing permission.

Reports should distinguish **Documented vendor guidance**, **Observed by CV Linter**, and, only when supported, **Observed in an authorized vendor test**, with tested scope available. Source review status, benchmark coverage, and observational confidence remain separate. A profile with one evaluated claim keeps its other claims documentation-based. No “ATS certified” badge, universal score, or implied vendor endorsement follows from a sandbox test or permission to publish.

## 6. Vendor research workflow and benchmark design

1. **Build the official source inventory.** For each priority vendor, collect parser/field, file-format/size, search, screening, API, partner/trial/sandbox, and applicable terms sources. Record date, feature/workflow scope, fact versus inference, and retrieval failures. Recheck unresolved Lever articles; ask vendors for missing parser contracts rather than filling gaps with career advice.
2. **Resolve access and permissions separately.** Confirm the real route (public parser, authenticated API, customer tenant, trial, partner sandbox, or dedicated test tenant), features, inspectable output, and eligibility. Obtain documented authorization for our synthetic fixtures and execution method; establish automation budgets, retention/cleanup, and publication rights independently. Do not apply to public jobs, probe undocumented services, or infer research rights from an API key.
3. **Preregister a small paired-fixture pilot.** Start with text-bearing PDF/DOCX baselines and controlled variants for columns, header contact information, native versus image text, Unicode/language, and field-to-record associations. Final fixture count is a proposal to agree with the vendor. Hidden-instruction fixtures require explicit inclusion in the study scope. Use fictional personas, controlled non-delivering contact details, and no real applications. Keep synthetic provenance in the manifest; inspect rendered fixtures and gold labels before submission.
4. **Establish the observable pipeline.** Record whether the chosen upload/API path actually triggers the target parser, completion/indexing signals and timeouts, feature configuration, and field origin. Capture raw parser output if available; otherwise label observations as profile/UI/export results. Control supplied fields, deduplication, enrichment, manual corrections, and record reuse. Opaque SaaS parser versions stay `vendor_version_unknown` with run time and available release identifiers.
5. **Measure each stage independently.** Record upload outcome, parse status, field precision/recall and associations, missing/duplicated text, and search retrieval against a preregistered query set after bounded indexing waits. Keep search terms out of unrelated notes/tags so broad search cannot mimic resume indexing. Screening is a separate optional, specifically authorized workflow with fixed criteria; do not infer it from parser or search results. Run local parsers against independent gold labels using the same bytes and report that as a separate comparison.
6. **Review and publish only the permitted scope.** Include fixture counts, family splits, exclusions, dates, tenant configuration, protocol, repeats where behavior is stochastic, uncertainty, unavailable observations, and observed-versus-inferred conclusions. Publication permission must cover the intended vendor naming, field extracts/screenshots, methods, fixtures, and findings. Record any review conditions or withheld results; do not cherry-pick favorable outcomes. Without permission, preserve permissible internal records and keep public vendor guidance documentation-based.
7. **Maintain claims over time.** Recheck sources before a profile release; retest after material parser/workflow changes when allowed. Expired/revoked permission prevents new experiments; it does not erase a dated historical observation. A changed source/configuration can require `needs_retest`; old reports retain their original evidence and scope.

**Release decision:** Local parser correctness and evidence/report separation can be validated without vendor access. Documentation-based vendor guidance may ship with conservative scope and unknowns. “Tested with vendor X” requires authorized, reviewed results and appropriate publication permission for exactly the named scope; vendor-wide certification remains unsupported. Neither partner onboarding nor the proposed pilot expands CV Linter into candidate ranking, screening decisions, or application submission.

## 7. Proposed Teamtailor support/partner request — unsent

**Suggested route:** Teamtailor support or the tech partner application route linked in the [official integration guide](https://support.teamtailor.com/en/articles/5477200-integrate-your-hr-recruitment-tool-with-teamtailor). Ask whether an independent candidate-facing research tool qualifies; do not imply that CV Linter is already a customer or approved integration partner.

**Subject:** Approved sandbox request for synthetic CV parsing compatibility research

Hello Teamtailor Support / Partnerships,

We are planning CV Linter, a local tool that helps individuals inspect how their CV's text and structure are extracted. We would like to assess a small, clearly scoped set of synthetic fixtures in an approved Teamtailor environment. We currently describe Teamtailor behavior only through official documentation and are not claiming tested compatibility or certification.

Could you advise whether the partner program or another research/test-tenant route is appropriate, and confirm the following in writing?

- An approved sandbox or dedicated test tenant, with the relevant resume parser and Co-pilot features enabled, plus confirmation of how its configuration differs from production.
- Permission to upload fictional PDF and DOCX fixtures through an agreed private workflow, including agreed image-based/layout variants. We would separately agree any hidden-instruction detection fixtures before using them.
- Inspectable parsed fields and field associations, including whether personal information, work history, education, skills/traits, and detection warnings are available directly or only through the candidate UI. Please distinguish parser output from summaries, enrichment, or manually supplied fields.
- API and/or export access for uploading fixtures and retrieving permitted results, required scopes, completion/indexing signals, and confirmation that the chosen route invokes the parser being evaluated.
- Permission for scripted synthetic testing, with agreed total uploads, batch size, concurrency, rate limits, retry rules, test dates, and a stop/contact procedure. Please identify applicable trial, customer, partner, and research terms or any exceptions needed.
- An isolated setup with no public job applications, candidate communications, or real candidate data, and agreed retention/deletion of test records.
- Permission to publish scoped methods and findings, including the allowed use of Teamtailor's name, synthetic fixtures, metrics, field extracts, and screenshots, with any confidentiality or prepublication review conditions stated explicitly. We would report limitations and avoid certification or hiring-outcome claims.

If this is outside the partner program, is there an approved alternative for this research, or additional official parser documentation we should use while our profile remains documentation-based?

Thank you,
CV Linter project

**Next action:** The project owner should send this reviewed request through the appropriate support/partner route, then record the response and permission scope. This document drafts the request only; it does not authorize or perform outreach. While awaiting access, continue the four-vendor source inventory and local fixture/annotation planning. Application implementation and fixture uploads remain future work requiring their own authorization.
