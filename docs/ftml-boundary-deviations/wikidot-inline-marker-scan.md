# Deviation: Wikidot compatibility inline marker scan

- Shim: `next_wikidot_compat_inline_marker` in
  `deepwell/src/services/render/wikidot_inline_markers.rs`, called by the
  compatibility fallback in `deepwell/src/services/render/service.rs`.
- Reason it lives in Wikijump: this change hardens the already-frozen BND-01
  fallback surface without adding syntax capability. Keeping the bounded scan
  beside that fallback preserves its literal-on-invalid behavior while making
  candidate discovery monotonic.
- Why FTML is not yet sufficient: FTML does not currently render every legacy
  Wikidot inline color, underline, and italic shape that reaches this fallback.
  Removing the scan before those gaps close would expose authored marker text.
- Evidence: PR #395 and its focused scanner, fallback-rendering, and dense
  adversarial regression tests.
- FTML backlog decision: no new grammar debt is accepted. This is a correctness
  and complexity fix within the existing BND-01/BND-04 cleanup backlog recorded
  by the 2026-07-09 FTML/Wikijump boundary audit.
- Migration condition: FTML's Wikidot layout covers the evidenced marker corpus
  and malformed-input behavior, and the pin-bump canary shows that the fallback
  branch can be removed without browser-visible regressions.
- Owner: Rokurolize/Wikijump maintainers.
- Review trigger: any FTML pin bump that changes inline color, underline, or
  italic parsing, or removal of the BND-01 compatibility fallback.
