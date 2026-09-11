# Rule configuration boundary

Read this reference when the user asks to configure, enable, disable, or change the severity or options of CV Linter rules, or when implementing that capability.

## Current behavior

General rule configuration is not implemented. Rules, severities, and thresholds come from the versioned Rust ruleset. The only per-run option is the bounded spelling `allow_words` input exposed by the CLI and MCP; it is recorded in the report and never persisted.

Do not claim that a configuration file, preset, rule override, or custom rule is currently honored. An agent may explain or propose the deferred design, but must not silently treat a requested override as applied.

## Deferred design

This is the host-facing summary of the deferred architecture TODO. The intended direction borrows ESLint's stable rule IDs, severity selection, and typed per-rule options while keeping configuration static and bounded.

An illustrative shape, not a committed file-format decision, is:

```toml
config_version = 1

[rules."cv.structure.dense_block"]
severity = "info"
max_characters = 1200
```

The resolved design should:

- support `off`, `info`, `warning`, and `error` without changing the meaning of other rules;
- validate rule-specific typed options and reject unknown rule IDs, severities, and option names;
- resolve configuration in the transport-independent Rust core so CLI and MCP remain equivalent;
- preserve built-in defaults and record the effective configuration or a deterministic hash in reports;
- require an explicitly selected configuration file or bounded inline input instead of scanning parent or home directories;
- reject executable configuration, unrestricted custom rule code, and initially unrestricted regular expressions;
- prevent an AI host from silently weakening rules selected by the user.

Before implementation, decide and document the static file format, CLI input, MCP input, precedence with per-run options, size limits, and versioning behavior. Acceptance tests should cover unchanged defaults, enable/disable and severity behavior, threshold boundaries, invalid or unknown configuration, CLI/MCP parity, and effective-config identity.
