# Browser typography compatibility audit (2026-10-11)

This report separates **independently measured current Wikidot values**, browser CSSOM differences, **pinned historical fixture semantics**, and **what is not yet established**. Wikidot—not modern Web styling preference—is the authority for legacy page behavior.

## Recorded current live Wikidot browser observations

Source: `https://scp-wiki.wikidot.com/theme:yossistyle`, HTTP 200 in Chromium, Firefox, WebKit and Pixel 7 Chromium; recorded as computed CSS using real Playwright browsers. Current live page is **YOSSISTYLE CSS THEME**, with text/InterVariable styling unlike the old XML-RPC fixture (`YOSSISTYLE`, Verdana body 0.8em). **Do not compare their absolute font sizes and call it source-equivalent parity.**

| Browser | Body font size | Paragraph font size | Title font size | Title/body | Paragraph/body |
|---|---:|---:|---:|---:|---:|
| chromium | 15.2px | 15.2px | 30.4px | 2.0 | 1.0 |
| firefox | 15.2031px | 15.2031px | 30.4063px | 2.00001 | 1.0 |
| webkit | 15.2px | 15.2px | 30.4px | 2.0 | 1.0 |
| mobile-chromium | 15.2px | 13.68px | 30.4px | 2.0 | 0.9 |

Full measured values and source/fixture limitations are in `typography-browser-matrix-20261011.json`. The raw live capture was made in an isolated remote WSL browser session (`/tmp/wj-1990-live-yossistyle-20261011.json`; raw SHA-256 pinned in JSON). It is not a claim of current live-reference content equivalence.

## Reproduction of previously untested browser failure

- Previously committed `wikidot-typography-parity.spec.ts` ran on a local plain-HTTP Vite fixture: **1/4 pass** when extended to Chromium, Firefox, WebKit, and mobile Chromium. Firefox CSSOM quantizes 12.8px to 12.7969px. WebKit upgrades CSS requests to `https://localhost` because of CSP `upgrade-insecure-requests`, but the HTTP fixture has no TLS, so vendored local CSS does not apply (title computed 12.8 instead of 25.6). Mobile Chromium uses a 0.9 paragraph font factor and should not be measured against a desktop-only absolute reference.
- When the already implemented TLS fixture is used, WebKit receives the three vendored stylesheets and the title returns to 25.6px. WebKit paragraph leading is represented as `19.200001px`, so exact decimal CSSOM text equality is also invalid.

## New positive control (test quality, not final Wikidot parity)

```sh
cd framerail
node tests/playwright-runner.js wikidot-typography-https.spec.ts --config playwright.wikidot-typography-https.config.ts --reporter=line
```

**PASS 4/4:** Chromium HTTPS, Firefox HTTPS, WebKit HTTPS, Pixel 7 Chromium HTTPS. This test checks loaded same-origin vendored stylesheets, source-authored font family/size and computed ratios (2x title, 1.5x paragraph leading, 0.9x mobile paragraph) while accommodating browser CSSOM decimal quantization. The ordinary default Chromium absolute-reference test is preserved. No production stylesheet or Wikidot behavior is changed.

Still required for final parity: independently acquire and preserve **identical authored page/theme source** in Wikidot and Wikijump, then compare desktop/mobile browser screenshots, CSSOM and pointer/keyboard/focus interactions, including browser version and viewport metadata. The current published theme page is not equivalent to the historical fixture. No claim of final-zero PASS, no standing/443 promotion.
