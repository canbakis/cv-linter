# CV Linter: Deferred UI Direction

**Date:** 10 September 2026
**Status:** UI deferred from MVP; no React, browser, PWA, desktop, or usability implementation is claimed.
**Related:** [Product and architecture plan](architecture-and-product-plan.md) · [Agent Skill and MCP architecture](agent-skill-architecture.md)

## 1. MVP experience

MVP interaction is terminal- and host-based:

1. The user selects one CV file.
2. `cv-linter lint` reports deterministic local findings.
3. The user may explicitly request `extract-text` or `extract_cv_text` for a configured Codex/Claude workflow.
4. The host performs semantic job-requirement evidence analysis or rewriting using cited extracted blocks.

The host must keep deterministic findings separate from semantic advice. Advice is not a score, hiring decision, or automatic edit. Unsupported or uncertain claims are marked as not evidenced or abstained, and every substantive host-AI claim cites stable block IDs.

## 2. Privacy copy

Use this concise disclosure in the skill and host setup:

> Parsing and deterministic linting run locally in the CV Linter executable. Extracted CV content explicitly handed to the configured AI host enters that host's context and follows the host/model's data policies. The complete workflow is local only when the selected model is local and verified not to forward data.

Do not describe the entire workflow as local merely because parsing is local. The executable has no raw-CV storage, database, account, cloud sync, analytics, or automatic report history. It holds selected bytes/text only in process memory or explicit user-directed output streams.

## 3. Deferred product surfaces

React/browser UI and PWA/offline UI are deferred until observed user needs justify a visual client. The earlier browser-first layout is retained only as a research hypothesis: non-agent users may value drag-and-drop, source highlighting, and visual extraction inspection, but this has not been validated.

Tauri or other desktop packaging is deferred until there is evidence from paying customers. Do not add a desktop shell, local HTTP bridge, always-running backend, or hidden localhost service to make a UI appear available.

Other deferred UX includes authentication, billing, cloud sync, Supabase CV storage, durable history, broad export, analytics, vendor profiles, ATS-compatible badges, scores, built-in judging, and automatic rewriting.

## 4. Future UI research questions

If evidence warrants a UI spike, first test the smallest visual surface against the existing CLI/MCP contracts:

- Can users select one intended CV and understand what was extracted?
- Can they distinguish a deterministic document finding from host-AI semantic advice?
- Can they follow a block citation back to the source and notice missing/uncertain extraction?
- Do Swedish users need capabilities that the parser and spelling spikes do not yet provide?
- Does a visual client create enough value to justify its browser or desktop packaging and privacy complexity?

Any future design must preserve the local parsing/host-processing distinction, show source locators and stable block IDs, avoid ATS-pass or hiring-success language, and keep raw CV content out of product-managed persistence by default.
