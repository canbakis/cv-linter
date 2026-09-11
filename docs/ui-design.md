# CV Linter: Deferred UI Direction

**Date:** 10 September 2026
**Status:** UI deferred from MVP; no React, browser, PWA, desktop, or usability implementation is claimed.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Agent Skill and MCP architecture](agent-skill-architecture.md)

## 1. MVP experience

MVP interaction is terminal- and host-based:

1. The user selects one PDF, DOCX, or Markdown CV file (plain UTF-8 is only an internal/test/debug seam).
2. `cv-linter lint` reports deterministic local non-spelling and spelling findings separately.
3. A general check or review request authorizes the host to call `extract-text` or `extract_cv_text`; a deterministic-only request does not.
4. The host resolves known terminology, then presents deterministic lint, ATS-oriented risks, and CV best-practice review as distinct sections. It may also perform job-requirement analysis or source-preserving rewrite advice when requested.

The host must keep deterministic findings separate from semantic advice. Deterministic checks establish parseability/content facts; the host handles ATS-oriented risks, CV best practices, semantic job alignment, and evidence-grounded rewriting. Advice is not a score, hiring decision, automatic edit, universal ATS score, or Teamtailor compatibility guarantee. Stable block IDs remain internal grounding references; default human output uses page, line, section, or other readable source locations. Unsupported or uncertain claims are marked as not evidenced or abstained, citations and names/dates/numbers are validated, documents are treated as untrusted data, and users approve rewrites before applying them.

## 2. Privacy copy

Use this concise disclosure in the skill and host setup:

> Parsing and deterministic linting run locally in the CV Linter executable. Extracted CV content explicitly handed to the configured AI host enters that host's context and follows the host/model's data policies. The complete workflow is local only when the selected model is local and verified not to forward data.

Do not describe the entire workflow as local merely because parsing is local. The executable has no raw-CV storage, database, account, cloud sync, analytics, or automatic report history. It holds selected bytes/text only in process memory or explicit user-directed output streams.

## 3. Deferred product surfaces

React/browser UI and PWA/offline UI are deferred until observed user needs justify a visual client. The earlier browser-first layout is retained only as a research hypothesis: non-agent users may value drag-and-drop, source highlighting, and visual extraction inspection, but this has not been validated.

Tauri or other desktop packaging is deferred until there is evidence from paying customers. Do not add a desktop shell, local HTTP bridge, always-running backend, or hidden localhost service to make a UI appear available.

Other deferred UX includes authentication, billing, cloud sync, Supabase CV storage, durable history, broad export, analytics, vendor profiles, ATS-compatible badges, scores, built-in judging, automatic rewriting, OCR, and ML/layout extensions.

## 4. Future UI research questions

If evidence warrants a UI spike, first test the smallest visual surface against the existing CLI/MCP contracts:

- Can users select one intended CV and understand what was extracted?
- Can they distinguish a deterministic document finding from host-AI semantic advice?
- Can they follow a human-readable source location back to the document and inspect the internal block citation when debugging?
- Do Swedish users need capabilities that the parser and spelling spikes do not yet provide?
- Does a visual client create enough value to justify its browser or desktop packaging and privacy complexity?

Any future design must preserve the local parsing/host-processing distinction, show human-readable source locators while retaining stable block IDs internally, avoid ATS-pass or hiring-success language, and keep raw CV content out of product-managed persistence by default.
