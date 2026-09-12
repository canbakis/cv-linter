---
name: setup
description: Set up the CV Linter executable required by this Claude plugin. Use when the cv-linter MCP server is missing or the user invokes setup explicitly.
disable-model-invocation: true
---

# Set up CV Linter

Check whether `cv-linter --version` succeeds. If it does, report the installed
version and ask the user to restart or reload the plugin if its MCP server is
still unavailable.

If it is missing, explain that the plugin needs the local CV Linter executable
on `PATH`. Offer the official GitHub Release installer for the user's operating
system as documented in the repository README, or `cargo install --git` for a
user who already has Rust. Show the exact command and ask for confirmation
before downloading or executing it. Do not silently install software, elevate
permissions, or weaken operating-system security controls.

After installation, run `cv-linter --version` and report the result. Do not read
or lint a CV during setup.
