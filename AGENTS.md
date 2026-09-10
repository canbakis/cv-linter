# Repository guidance

- Treat [docs/architecture-and-product-plan.md](docs/architecture-and-product-plan.md) as the canonical product scope.
- Route command and MCP details to [docs/agent-skill-architecture.md](docs/agent-skill-architecture.md), UI decisions to [docs/ui-design.md](docs/ui-design.md), and research-only evidence to `docs/research/`.
- Preserve the MVP boundary: one Rust core with thin CLI and stdio-MCP entry points. Keep static, byte-derived linting in Rust and semantic interpretation in the configured host; do not imply that local parsing makes host/model processing local.
- Apply YAGNI. Keep claims qualified by their evidence and distinguish hypotheses, local observations, and authorized vendor evidence.
- `src/lib.rs` owns the transport-independent core; `src/main.rs` is the thin CLI adapter. Keep business logic in the core for future MCP/UI adapters.
- When subagents are available, delegate clearly bounded documentation work, repository inventories, formatting, JSON validation, and simple mechanical tasks to `luna_worker`, which uses `gpt-5.6-luna`. Multiple Luna agents are appropriate only for genuinely independent, read-heavy work; serialize overlapping writes. The primary agent reviews and integrates all delegated output.
- Keep ambiguous architecture, security/privacy, licensing/legal, destructive actions, and major product decisions with the primary agent.
- Current Rust validation commands are `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`. Documentation checks are `jq empty docs/research/*.json` and `git diff --check`.
- Preserve unrelated dirty-worktree changes exactly. Do not commit unless explicitly asked.
