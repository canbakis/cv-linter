# Bundled spelling dictionaries

This directory vendors unmodified dictionary and notice files from the
LibreOffice `dictionaries` repository at commit
`32b006a2c22a4ac7e8ed3f03346f7b3d85a970a4` (retrieved 2026-09-11).
The source paths are:

- `sv_SE/dictionaries/sv_SE.aff`
- `sv_SE/dictionaries/sv_SE.dic`
- `en/en_US.aff`
- `en/en_US.dic`

The dictionary bytes were verified after download:

| Bundled file | SHA-256 |
|---|---|
| `sv_SE/sv_SE.aff` | `b721c9d44bee912feb182b601a1bc2ae3e7dffef660f4130cf2751867488a9dd` |
| `sv_SE/sv_SE.dic` | `384a2126eff333f5f6f9790ae892554546f53948d2988c600397cb5ad6ce66e8` |
| `en_US/en_US.aff` | `e746c882dd6f303c2c46e7452804b9201115a6942cfeb15f18f8edf774d2e24e` |
| `en_US/en_US.dic` | `f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647` |

The original Swedish notices are preserved as `LICENSE_en_US.txt` and
`LICENSE_sv_SE.txt`. The original English notices and provenance are preserved
as `README_en_US.txt`, `README.txt`, `WordNet_license.txt`, and `license.txt`.
Those upstream files describe multiple sources and terms; this inventory is
provenance, not legal advice or assurance that a particular distribution model
meets every obligation. Spellbook is a separate Cargo dependency distributed
under MPL-2.0 and is not copied into this directory.
