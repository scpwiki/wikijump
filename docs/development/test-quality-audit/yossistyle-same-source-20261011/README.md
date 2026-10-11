# Same-source yossistyle browser comparison: bounded evidence (2026-10-11)

Tracking: #1990, #2325, #2330. The 2026-07-13 XML-RPC typography fixture is NOT the source used in this comparison. Do not mistake the 4/4 narrow computed-style result for a full-page parity PASS.

## Source / environment identity

- Source: shared corpus `en/pages/theme:yossistyle/source.wikidot.txt`, fetched in run `run-20260816-100230` (corpus metadata page revision count 9, last modified 2021-09-11). SHA-256 `561228038e46e76ab45e61d95570e82519bceea1aa5233addbbec324601ca4d4`; exact bytes are retained as `source.wikidot.txt.gz`.
- Current local Wikijump mirror: `https://scp-wiki.wikijump.localhost/theme:yossistyle`, read-only, HTTP 200. The same authored wikitext (7,723 characters) is present verbatim in its saved SSR HTML (`local-rendered.html.gz`, losslessly compressed); `verify.mjs` asserts the source SHA matches the corpus file. This DOES NOT prove that the live Wikidot revision remained unchanged after the August corpus acquisition.
- Independent Wikidot reference: `https://scp-wiki.wikidot.com/theme:yossistyle`, 2026-10-11 live browser observation saved by preceding audit in `live-wikidot-observation.json` (SHA-256 `632948f633aea578b55225958521c1f2607e17be2f4db811be08e`). `local-wikijump-observation.json` is a separate 2026-10-11 measurement. This run did not recontact Wikidot.
- Browsers: Playwright 1.63.0 Chromium, Firefox, WebKit, Pixel 7 Chromium; desktop 1280x800, mobile 412x839. 2026-10-11 JST. Local browser screenshot files are retained. No equivalent live screenshots, resource bytes or full DOM snapshots were available from the previous Wikidot acquisition; therefore neither pixel nor full-document parity is established. Browser executable hash and font-file identity were not captured in the earlier Wikidot observation.
- The local capture used `domcontentloaded` followed by 2.5s; the earlier live capture may have different font/load timing. Body-height comparisons are diagnostics only until timing and loaded-asset equivalence are secured.

## Measured comparison

| Engine | First paragraph / page-title text, computed CSS and geometry | HTML `<title>` live / local | Local minus live body height | Stylesheet links live / local | Local request failures |
|---|---|---|---:|---:|---:|
| Chromium | EXACT | `YOSSISTYLE CSS THEME - SCP Foundation` / `YOSSISTYLE CSS THEME \| SCP Foundation` | +1 px | 5 / 8 | 0 |
| Firefox | EXACT | same differing separator | +1 px | 9 / 8 | 1 (favicon.gif `NS_ERROR_DOM_NETWORK_ERR`) |
| WebKit | EXACT | same differing separator | +1 px | 5 / 8 | 0 |
| Mobile Chromium | EXACT | same differing separator | +144.71875 px | 8 / 8 | 0 |

All engines report the same first-paragraph font family, size, line-height, text-rendering, `#page-title` geometry, paragraph geometry, and body computed typography when comparing reference/candidate *within each engine*. The source of the outer-body height discrepancy is not yet isolated. The local site's CSS links do not have the same count as the previous live capture; do not assume identical stylesheet/dependency bytes or load timing.

**Reproduced public-browser discrepancy:** the browser tab title uses ` - ` on live Wikidot but ` | ` on local Wikijump; local source owner: `framerail/src/routes/[slug]/[...extra]/PageHead.svelte` currently emits `<title>{title} | {siteName}</title>`. Recorded as #2330; this warrants additional page samples and additional page/site samples before imposing a general title rule. The first paragraph and HTML page heading do match; the delimiter affects the document title, not `#page-title`.

## Reproducibility, limits and next steps

Run `node docs/development/test-quality-audit/yossistyle-same-source-20261011/verify.mjs` (offline). For an explicit repeat of the **local-only** browser capture, run `node docs/development/test-quality-audit/yossistyle-same-source-20261011/capture-local.cjs /absolute/scratch-output-dir` from the repository checkout (requires the Framerail Playwright install and a functioning local mirror). The capture script does not contact Wikidot. It validates the retained source SHA against the mirror SSR embedded source, the Wikidot capture identity, and each engine's matched typography/rectangles. It reports whole-page parity as UNVERIFIED. It intentionally encodes the observed tab-title mismatch instead of making it pass.

Next, acquire current live source SHA and full browser HTML/CSS/resource bundle via explicit cache-sealed acquisition. Compare that with the local mirror's actual source, applied stylesheets, DOM structure, full-page rectangles and same-version screenshots. In particular, isolate the mobile 144.71875px body-height delta, favicon asset failure, and browser title delimiter. No standing/443 promotion or global final-zero claim is justified by these receipts.
