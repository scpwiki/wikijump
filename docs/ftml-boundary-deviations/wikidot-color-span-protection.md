# Deviation: Wikidot color-span protection

- Shim: `RenderService::protect_wikidot_color_spans` and `RenderService::restore_protected_wikidot_color_spans` in `deepwell/src/services/render/service.rs`, using `CompatHtmlFragments` and `LiteralRegionIndex`.
- Reason it lives in Wikijump: This is a bounded correctness and denial-of-service fix for the existing BND-08 compatibility shim; it does not add or broaden color syntax.
- Why FTML is not yet sufficient: The pinned FTML renderer does not preserve the corpus-backed Wikidot `##color|body##` forms needed by the existing compatibility path.
- Evidence: PR #397 audit; focused regressions `protects_colors_only_outside_authored_literal_and_attribute_regions`, `restores_registered_colors_only_in_rendered_html_text_nodes_linearly`, and `context_aware_restore_only_expands_markers_in_html_text_nodes`.
- FTML backlog decision: BND-08 in `docs/ftml-boundary.md` already tracks color-span protection as FTML-candidate debt; this change only hardens that frozen surface.
- Migration condition: FTML parses and renders the evidenced color forms under `Layout::Wikidot`, including literal-region behavior, and the required pin-bump marker canary passes.
- Owner: Wikijump rendering maintainers.
- Review trigger: Every FTML pin bump, or any change to BND-08 color parsing/restoration.
