# Issue #1990 — visual source inventory gap (2026-10-11)

The frontier inventory previously accepted JS/TS/Svelte but ignored `.scss`, `.css`, `.sass` and `.html` source. **Wikidot parity includes layout, styling, document shell, and interaction details even when their original behavior may be undesirable by general web-development standards.** The only change to rendering here is **none**: this is inventory and audit reachability, not a CSS or rendering rewrite.

From an isolated `504f2f5883cf` develop checkout, adding visual production extensions changed the frontier inventory from **1,309 to 1328** files. 19 visual sources (18 SCSS + one HTML shell) were newly accounted for. The scan listed 876 executable test-owner candidates, with **zero static JS-import owner edges** detected for these visual sources; that is **not** evidence of untested CSS, because Sass and Svelte/Vite dependency transformations are not modeled by the static importer.

## Omitted tracked visual production surfaces

| Visual production source | SHA-256 prefix |
|---|---|
| `framerail/src/app.html` | `f4f5b16332c3f277` |
| `framerail/src/lib/css/_abstracts.scss` | `22e1bcb25da9f58b` |
| `framerail/src/lib/css/abstracts/_functions.scss` | `0b226eff994c7d81` |
| `framerail/src/lib/css/abstracts/_media.scss` | `0bf024c16443757e` |
| `framerail/src/lib/css/abstracts/_mixins.scss` | `fb6900044c9b56b8` |
| `framerail/src/lib/css/abstracts/_variables.scss` | `1e3de67846caa5b6` |
| `framerail/src/lib/css/base/_a11y.scss` | `0db74b7896176e0c` |
| `framerail/src/lib/css/base/_base.scss` | `4c2dddfcf50a6461` |
| `framerail/src/lib/css/base/_font-faces-cjk.scss` | `ee47bd091cff1b3d` |
| `framerail/src/lib/css/base/_font-faces.scss` | `86f0bd81662a028e` |
| `framerail/src/lib/css/base/_reset.scss` | `77414cd0ebc00c8a` |
| `framerail/src/lib/css/base/_typography.scss` | `159911f2c44a4dbd` |
| `framerail/src/lib/css/main.scss` | `3a7658482da2299a` |
| `framerail/src/lib/css/themes/_code.scss` | `d349d50cddca5630` |
| `framerail/src/lib/css/themes/_colors.scss` | `2231117d1362e904` |
| `framerail/src/lib/css/themes/_layout.scss` | `2cc4a7f24f36d3af` |
| `framerail/src/lib/css/themes/_rhythm.scss` | `3eb716af3856a58f` |
| `framerail/src/lib/css/themes/_typography.scss` | `cf9a49f9bc2e6951` |
| `framerail/src/routes/[slug]/[...extra]/page.scss` | `ae4617253dd663a4` |

## Reproduction and acceptance

Run in a checkout with all scoped production sources available:

```sh
node --test install/local/wikidot-verification/tests/test-quality-audit.test.mjs
node scripts/run-test-quality-frontier-audit.mjs inventory --output-dir /tmp/unique-new-directory
node scripts/verify-test-quality-evidence-identities.mjs
```

The executable regression compares **all git-tracked `.css/.scss/.sass/.html` sources under `framerail/src`** with actual frontier production rows, rather than accepting a hand-selected sample; it also rejects counting style/HTML files as executable test owners. Current result: **9/9 suite PASS**, identity verifier **PASS**.

This report is not a browser rendering test, a mutation disposition, a Wikidot compatibility conclusion, or final-zero. To decide whether visual sources actually preserve Wikidot, review computed CSS, screenshots and keyboard/pointer interactions with same-page independent Wikidot evidence across responsive viewport and browser combinations. Open/unknown source ownership stays unapproved; do not replace quirks with modern-web design preferences.

The machine-readable row names and full source hashes appear in `visual-source-frontier-20261011.json`.
