# Spellbook integration validation

Status: local MVP evidence record, 2026-09-11.

This note separates three kinds of information:

- **Upstream evidence** is metadata or behavior documented by Spellbook or the
  LibreOffice dictionary repository.
- **Local observation** is what the pinned files, implementation, and focused
  repository checks show on this checkout.
- **Open review** identifies licensing or compatibility questions that this
  engineering validation does not settle.

Nothing in this note is legal advice or an assurance that a particular release
or distribution model satisfies every license obligation.

## Pinned Spellbook integration

`Cargo.toml` pins the published crate and its only requested feature exactly:

```toml
spellbook = { version = "=0.4.2", default-features = false, features = ["default-hasher"] }
```

As upstream evidence, the
[`0.4.2` crate metadata](https://crates.io/crates/spellbook/0.4.2) declares:

- Rust edition 2021 and `rust-version = "1.70"`;
- `license = "MPL-2.0"`;
- an unconditional `hashbrown >=0.15` dependency;
- one default feature, `default-hasher`, which enables the optional
  `foldhash >=0.1` dependency; and
- a `no_std` library that requires allocation but performs no document I/O or
  tokenization.

Disabling future default features and then explicitly enabling
`default-hasher` keeps the intended feature set visible while retaining
`Dictionary::new`. `Cargo.lock` resolves `spellbook 0.4.2` with crate checksum
`0d204abcbdf8e88729306a8d0ca01a79d8a49969fe1f84696909d6dbc4321c1a`.
The signed `v0.4.2` tag resolves to source commit
`2d278335f592defadeb1133917481bf510b5e635`.

The relevant upstream API is:

```rust
Dictionary::new(aff: &str, dic: &str)
    -> Result<Dictionary, ParseDictionaryError>
Dictionary::check(&self, word: &str) -> bool
```

Spellbook also exposes `Dictionary::checker`, `Dictionary::suggest`,
`Dictionary::suggester`, and mutable `Dictionary::add`. The MVP deliberately
uses only `new` and `check`: casing is not broadened with
`Checker::check_lower_as_title` or `check_lower_as_upper`, suggestions are not
generated, and allowlisted terms are not inserted into either dictionary.
Spellbook documents a maximum checked word length of 360 bytes and explicitly
delegates prose tokenization to its caller.

`cargo tree -e features -i spellbook` locally shows only the
`default-hasher` feature requested by `cv-linter`. This is evidence for the
current lockfile, not a statement about a future Spellbook release.

## Dictionary provenance and integrity

The four dictionary files are unmodified copies from
[`LibreOffice/dictionaries` commit
`32b006a2c22a4ac7e8ed3f03346f7b3d85a970a4`](https://github.com/LibreOffice/dictionaries/tree/32b006a2c22a4ac7e8ed3f03346f7b3d85a970a4).
The exact local observations are:

| Locale | Bundled path | Upstream path | Bytes | SHA-256 |
|---|---|---|---:|---|
| English (US) | `dictionaries/en_US/en_US.aff` | `en/en_US.aff` | 3,205 | `e746c882dd6f303c2c46e7452804b9201115a6942cfeb15f18f8edf774d2e24e` |
| English (US) | `dictionaries/en_US/en_US.dic` | `en/en_US.dic` | 551,762 | `f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647` |
| Swedish (Sweden) | `dictionaries/sv_SE/sv_SE.aff` | `sv_SE/dictionaries/sv_SE.aff` | 18,583 | `b721c9d44bee912feb182b601a1bc2ae3e7dffef660f4130cf2751867488a9dd` |
| Swedish (Sweden) | `dictionaries/sv_SE/sv_SE.dic` | `sv_SE/dictionaries/sv_SE.dic` | 2,344,202 | `384a2126eff333f5f6f9790ae892554546f53948d2988c600397cb5ad6ce66e8` |

The raw dictionary payload totals 2,917,752 bytes, about 2.78 MiB. Both affix
files declare `SET UTF-8`; the Swedish dictionary contains UTF-8 Swedish text,
while the current English dictionary file itself is ASCII-compatible.
LibreOffice's Swedish package metadata identifies version 2.42. The English
`README_en_US.txt` identifies its SCOWL-derived word-list provenance as
2020.12.07 and size 60; that embedded provenance date is not the version of the
later LibreOffice repository commit or every subsequent affix-file change.

## Preserved notices and open licensing review

The original source notices are retained alongside the data:

- Swedish: `dictionaries/sv_SE/LICENSE_en_US.txt` and
  `dictionaries/sv_SE/LICENSE_sv_SE.txt`;
- English: `dictionaries/en_US/README_en_US.txt`,
  `dictionaries/en_US/README.txt`,
  `dictionaries/en_US/WordNet_license.txt`, and
  `dictionaries/en_US/license.txt`.

As upstream evidence, the Swedish notices identify Göran Andersson as the
maintainer/copyright holder and say that the dictionary is available under
GNU LGPL version 3. The English provenance is composite: its retained files
describe SCOWL and Kevin Atkinson terms, Ispell/Geoff Kuenning terms, WordNet
terms, public-domain inputs, and other attribution or redistribution notices.
The presence of a GPL license text in the English package does not by itself
reduce all of that data provenance to one confidently inferred SPDX label.

Spellbook's MPL-2.0 metadata, the Swedish dictionary notice, and the English
source notices are independent inputs. The workspace's root MIT declaration
does not replace them. Before distributing a binary or source archive, an open
legal review must determine the required full license texts, attribution,
source availability, modification marking, packaging, and replacement or
relinking arrangements for the intended channel. The preserved notices and
this inventory support that review; they do not complete it.

## Implemented deterministic behavior

The transport-independent implementation in `src/spelling.rs` currently does
the following:

1. It embeds the four pinned files with `include_str!` and parses one immutable
   English and one immutable Swedish `Dictionary` in a process-wide
   `LazyLock`. Focused tests fail if either bundled pair no longer parses.
2. It visits extracted blocks and candidate tokens in source order. Whitespace
   first separates chunks. Email- and URL-like chunks are skipped.
3. Tokens retain Unicode alphanumeric characters and combining marks. ASCII
   apostrophe, typographic apostrophe (`U+2019`), hyphen-minus, Unicode hyphen
   (`U+2010`), and non-breaking hyphen (`U+2011`) remain only as internal
   connectors. Original block-relative UTF-8 byte spans are retained as
   finding evidence.
4. Tokens with any numeric character, no alphabetic character, or more than
   Spellbook's 360-byte maximum are skipped.
5. A lowercase copy is used for allowlist membership and an additional
   dictionary lookup. A term is accepted when it is in the small versioned
   developer allowlist, in the operation-scoped user allowlist, or recognized
   in either original or lowercase form by **either** `en_US` or `sv_SE`.
   There is no language detector. This bilingual OR rule intentionally permits
   mixed-language CVs, with the tradeoff that a misspelling that is a valid
   word in the other dictionary is accepted.
6. Terms are deduplicated by their lowercase form before dictionary lookup,
   avoiding repeated checks for common or repeated misspelled words. Only the
   first unknown occurrence is emitted, so findings remain in first-seen
   block/token order. Issue collection is capped against the global 10,000
   finding limit.
7. An unknown token whose first alphabetic character is uppercase becomes the
   informational `cv.spelling.possible_name_or_term` finding. This is a
   deterministic low-confidence casing heuristic, not a semantic assertion
   that the token is a proper noun. Other unknowns become warning-level
   `cv.spelling.unknown_word` findings.
8. CLI `--allow-word WORD` may be repeated; MCP `lint_cv` accepts an
   `allow_words` array. At most 256 single alphabetic spelling tokens of at
   most 360 bytes are accepted; surrounding punctuation, whitespace, and
   digits are rejected. They use the same lowercase conversion for that
   operation and are not persisted, silently inferred, or added to
   Spellbook. The normalized applied set is sorted, deduplicated, and emitted
   as `options.spelling_allow_words` so lint artifacts remain reproducible.
9. The linter emits no correction suggestions and performs no edits. This also
   avoids depending on Spellbook's suggestion ordering or its more expensive
   default n-gram search.

The JSON ruleset identity records engine `spellbook`, engine version `0.4.2`,
the LibreOffice source commit, developer allowlist version `0.1.0`, and
matching policy `en_US_or_sv_SE`.

## Known Swedish compound risk

Spellbook's upstream README calls the crate alpha, says `check` works well for
the relatively simple `en_US` dictionary, and qualifies support for languages
using complex compounding directives. The bundled Swedish affix file uses
`COMPOUNDRULE`, `CHECKCOMPOUNDTRIPLE`, `SIMPLIFIEDTRIPLE`,
`CHECKCOMPOUNDDUP`, `CHECKCOMPOUNDREP`, and `NOSPLITSUGS`.

The Spellbook
[`master` changelog](https://github.com/helix-editor/spellbook/blob/119625e5b6fb374e818e2372c8860aa7202e16bb/CHANGELOG.md)
records an unreleased fix after 0.4.2 for incorrect acceptance of some
`SIMPLIFIEDTRIPLE` compound non-words. It also records an unreleased
`NOSPLITSUGS` suggestion fix. The latter is dormant because this MVP does not
request suggestions; the former means the released checker can still produce
false negatives for applicable Swedish compound cases. Successful parsing and
representative word tests do not establish full Swedish Hunspell parity.

The project therefore keeps the published exact pin, reports spelling as a
possible issue rather than an authoritative language judgment, and requires
representative Swedish compound regression fixtures before making broader
quality claims. A future released upgrade should be evaluated and pinned
explicitly rather than following Spellbook's moving branch.

## Focused reproduction checks

The following commands reproduce the metadata, feature, integrity, directive,
and behavior observations without modifying source files:

```sh
cargo info spellbook@0.4.2 --verbose
cargo tree -e features -i spellbook
sha256sum \
  dictionaries/en_US/en_US.aff dictionaries/en_US/en_US.dic \
  dictionaries/sv_SE/sv_SE.aff dictionaries/sv_SE/sv_SE.dic
wc -c \
  dictionaries/en_US/en_US.aff dictionaries/en_US/en_US.dic \
  dictionaries/sv_SE/sv_SE.aff dictionaries/sv_SE/sv_SE.dic
rg '^(COMPOUNDRULE|SIMPLIFIEDTRIPLE|NOSPLITSUGS|CHECKCOMPOUND)' \
  dictionaries/sv_SE/sv_SE.aff
cargo test --lib spelling::tests --quiet
cargo test --test formats \
  bilingual_spelling_is_located_deduplicated_and_allowlisted_per_run --quiet
cargo test --test cli cli_spelling_allow_words_are_operation_scoped --quiet
cargo test --test cli \
  stdio_mcp_accepts_bounded_operation_scoped_spelling_allow_words --quiet
git diff --check
```

These are focused synthetic checks of the implemented contract. They do not
replace a representative bilingual CV corpus, cross-platform packaging tests,
an independent dependency audit, or legal review.
