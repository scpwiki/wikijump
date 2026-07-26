# Deviation: Rate module footnote literal protection

Shim: `RenderService::expand_rate_modules_with_registry` in `deepwell/src/services/render/runtime_modules.rs`

Reason it lives in Wikijump: Wikijump owns Rate module runtime output. Its current pre-FTML expansion must avoid inserting a block widget into a footnote, where live Wikidot preserves the module source as text.

Why FTML is not yet sufficient: The current runtime integration expands Rate from raw source before FTML provides container context or an injectable runtime module handle. The existing compatibility text registry is therefore used to carry the evidenced literal through FTML safely.

Evidence: `/home/roku/wjlab/evidence/syntax-differential-20260726/runtime-pages-v3/runtime-preview-references-all.jsonl`, source SHA-256 `be2900a14626833c8d331f409c9b12826b955f5d766ab1ac62c4a78518abe781`, raw live HTML SHA-256 `3755301728ea61d84703169eb220caa9aaea3c5b291faaa0c9fdb9f5e31a1fbe`

FTML backlog decision: This is newly accepted Wikijump-side debt until the architecture review for an injectable FTML runtime handle can represent module placement context without a raw-source prepass.

Migration condition: FTML must preserve Rate as a delayed module with its container context and let Wikijump resolve it only where Wikidot executes it.

Owner: Rokurolize

Review trigger: The FTML runtime-handle architecture review, any replacement of the Rate prepass, or 2026-10-26, whichever occurs first.
