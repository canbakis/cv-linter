# Xberg integration validation

Status: local MVP evidence record, 2026-09-11.

This note records what is currently verified in the repository. It is not a
claim that Xberg or every transitive dependency is suitable for every
deployment or licensing scenario; those questions need a separate review.

## Pinned integration

`Cargo.toml` pins `xberg = "=1.1.5"` with default features disabled and only
these requested features enabled:

```toml
features = ["tokio-runtime", "pdf", "office"]
```

The project-owned adapter in `src/lib.rs` passes an in-memory
`ExtractInput::from_bytes` value to Xberg. It does not pass a path or URL to
the dependency. The extraction configuration currently sets:

- `use_cache: false`;
- `disable_ocr: true`;
- `include_document_structure: true`;
- page extraction enabled through `PageConfig`;
- a 30-second cooperative extraction timeout and 10 MiB embedded-file cap;
- explicit `SecurityLimits`: 50 MiB expanded archives, 10 MiB extracted
  content, 1,000 archive files, nesting/XML depth 128, 1,000,000 iterations,
  10,000 table cells, and 100 PDF pages.

The project rejects Markdown above 10,000 logical lines before calling Xberg
and stops project-owned adaptation above 10,000 output blocks. The timeout is
not a hard process kill: synchronous parser work that does not yield cannot be
forcibly preempted. Input, expansion, iteration, page, and output caps are
therefore the primary resource controls.

The adapter accepts `.pdf`, `.docx`, `.md`, and `.markdown`; `.txt` remains an
explicit plain-UTF-8 test/debug seam. PDF and DOCX use Xberg; Markdown uses the
same adapter boundary and derives source byte locations from the original
input where possible. The feature tree inspected with `cargo tree -e
features -i xberg` shows the expected Xberg feature closure (including
`pdf-native` and the format dependencies implied by `office`); no OCR feature
is requested.

The resolved feature tree did not contain `reqwest`, `ureq`, or `hf-hub`.
That is evidence about this build's dependency graph, not a guarantee about
future Xberg versions or runtime behavior outside this adapter.

## Test evidence

`cargo test --test formats --quiet` covers:

- deterministic Markdown extraction and source-byte locators;
- PDF extraction with page locators;
- DOCX extraction with element locators;
- blank/image-only PDF behavior and the explicit no-OCR warning;
- unsupported extensions, malformed PDF/DOCX, and password-protected PDF
  errors;
- deterministic lint rules and ruleset identity;
- pre-extraction Markdown limits and exact-or-unknown Markdown byte locators.

`tests/cli.rs` additionally exercises CLI extraction for all three MVP native
formats and compares CLI JSON with the stdio-MCP JSON for Markdown extraction
and linting. Fixtures are synthetic and repository-owned (`tests/common/`),
so these tests do not establish broad corpus compatibility.

## Toolchain, license, and footprint observations

The local registry manifest for `xberg 1.1.5` declares `rust-version = "1.92"`
and `license = "MIT"`; the workspace currently builds with Rust 1.94.0. The
workspace package itself declares MIT. These are manifest observations, not a
complete legal review of the full transitive dependency set.

`Cargo.lock` contained 424 package records after adding the adapter; the
current full MVP lockfile contains 426 after the separate Spellbook addition.
The current runtime dependency tree has nine direct crates, including Xberg.
An Xberg-only measurement before spelling integration produced a 39,823,232-byte
unstripped x86-64 Linux release binary and a 32,406,088-byte manually stripped
copy. The current full MVP, including embedded spelling dictionaries and the
configured release symbol stripping, measures 35,680,304 bytes. One warm
full-lint run against the 492-byte Markdown fixture completed in 0.03 seconds
with a measured maximum resident set of 28,800 KiB. These are observations
from one machine and synthetic fixture, not cross-platform packaging or
performance guarantees or a way to attribute every byte to Xberg.

## Reproduction commands

```sh
cargo tree -e features -i xberg
cargo test --test formats --quiet
cargo test --test cli --quiet
git diff --check
```
