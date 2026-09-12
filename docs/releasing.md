# Releasing CV Linter

CV Linter follows Semantic Versioning and uses annotated `vMAJOR.MINOR.PATCH`
Git tags. `Cargo.toml` is the product-version source of truth. The Claude plugin
and MCPB manifests carry the same version and are checked in CI.

Before 1.0, increment:

- `PATCH` for backward-compatible fixes and documentation;
- `MINOR` for new behavior or a deliberate breaking change to the CLI, MCP
  tools, or installation contract.

The JSON schema and deterministic ruleset have their own reported versions.
Change those when their respective contracts change; they do not replace the
package version.

## GitHub repository gate

Private vulnerability reporting and dependency vulnerability alerts were
enabled on 12 September 2026. Before the first release, also decide whether to
protect `main` with a repository ruleset requiring the `validate`, `licenses`,
and `security-audit` CI jobs. This is intentionally not enabled automatically:
requiring pull requests or checks changes the maintainer's merge workflow.

Add a short repository description and topics before announcing the project so
the GitHub landing page explains what the tool does. These are discoverability
settings and do not affect the build.

## Release artifacts

Pushing a matching version tag starts the generated `dist` workflow. It creates
a GitHub Release containing:

- native archives for Apple Silicon macOS, Intel macOS, ARM64 Linux, x64 Linux,
  and x64 Windows;
- shell and PowerShell installers;
- SHA-256 checksums, a machine-readable dist manifest, source archive, and
  GitHub artifact attestations;
- one `cv-linter.mcpb` Claude Desktop extension containing both macOS binaries
  and the x64 Windows binary.

The MCPB uses a dependency-free Node.js launcher to choose the bundled Rust
binary for the current platform and start `cv-linter mcp --stdio`. Claude
Desktop supplies the Node.js runtime. CV parsing and linting remain in Rust.

GitHub checksums and attestations establish artifact integrity and build
provenance. The first release is not yet Apple-notarized or Windows code-signed,
so operating-system download warnings remain possible. Do not describe these
artifacts as signed until those credentials and CI steps are configured and
tested. MCPB signing is also disabled until its current toolchain behavior is
validated end to end with Claude Desktop.

## Cut a release

1. Update the version in `Cargo.toml`, `.claude-plugin/plugin.json`, and
   `packaging/mcpb/manifest.json`.
2. Move the relevant changelog entries from `Unreleased` into a dated version
   section and update the comparison links.
3. Run the release checks:

   ```sh
   cargo install cargo-about --version 0.9.2 --locked --features cli
   cargo install cargo-audit --version 0.22.2 --locked
   cargo about generate about.hbs --locked --fail \
     --output-file THIRD-PARTY-LICENSES.html
   cargo audit
   scripts/check-release-version.sh
   cargo fmt --check
   cargo test --locked
   cargo clippy --locked --all-targets -- -D warnings
   jq empty docs/research/*.json .claude-plugin/plugin.json .mcp.json \
     packaging/mcpb/manifest.json
   npx --yes @anthropic-ai/mcpb@2.1.2 validate packaging/mcpb/manifest.json
   claude plugin validate . --strict
   dist generate --check
   git diff --check
   ```

   Review the generated third-party license inventory rather than treating a
   successful command as legal approval. Commit it whenever `Cargo.lock`
   changes. The original dictionary notices are maintained separately under
   `dictionaries/` and are included in every native archive and MCPB.

   `cargo audit` currently emits the informational
   `RUSTSEC-2024-0436` warning for the unmaintained `paste` crate. It is a
   proc-macro used transitively in pinned Xberg's dependency graph, has no
   patched release, and is not a reported vulnerability. It is not linked into
   the distributed executable. The project audit policy keeps this visible
   while failing vulnerabilities, unsound packages, and yanked packages.
   Re-evaluate it whenever Xberg is updated.

4. Confirm GitHub private vulnerability reporting is enabled and its link in
   `SECURITY.md` works while signed out.
5. Commit the release preparation and push `main`.
6. Create and push an annotated tag:

   ```sh
   git tag -a v0.1.0 -m "CV Linter 0.1.0"
   git push origin v0.1.0
   ```

7. Watch both the normal release jobs and the final Claude Desktop bundle job.
   Download the produced assets, verify their checksums/attestations, and smoke
   test the CLI and MCPB on each supported operating system before announcing
   the release.

Never move or reuse a published tag. If a release contains a product defect,
fix it in a new patch release. A failed GitHub Actions job that produced no
changed source or tag may be rerun from GitHub.

## Later distribution channels

The crate package is small enough for crates.io and the `cv-linter` crate name
was unclaimed during the initial readiness check, but publishing is deliberately
not part of the tag workflow yet. Add crates.io only after ownership, token
handling, package contents, and the third-party license inventory have received
a release review.

Homebrew and WinGet/Scoop can improve discoverability after the GitHub artifacts
have proven stable. Each adds another package definition and update path, so the
first release keeps GitHub as the single binary source of truth.
