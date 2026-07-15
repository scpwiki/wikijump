# Deviation: Wikidot fallback code metadata

## Shim

`scan_compat_code_blocks` in `deepwell/src/services/render/compat_fallback_code.rs` is the temporary BND-01 hand parser for complete line-oriented Wikidot `[[code]]` blocks reached by the oversized compatibility fallback. It preserves source order, body text, and the `type` and `name` metadata required to materialize hosted `/local--code/` resources.

## Reason it lives in Wikijump

Deepwell persists rendered code blocks through `TextBlockService` and serves them from runtime-owned `/local--code/` routes. The BND-01 fallback previously emitted a visual code block while discarding that metadata, so an imported component could render documentation but lose the CSS or JavaScript resource referenced by an include consumer. Retaining the metadata at this runtime boundary prevents that data loss without teaching the normal Deepwell render path a theme slug, component slug, or other content-specific rule.

## Why FTML is not yet sufficient

FTML exposes parsed `CodeBlock` values on successful parses, but the BND-01 oversized fallback is entered precisely when the full input is not sent through the normal FTML parse. FTML does not yet offer a delayed code-block structure that can be extracted conservatively from such preserved input and later evaluated or persisted by the caller.

## Fail-closed contract

The scanner accepts only complete, non-nested line-oriented blocks with quoted attributes. Attribute names are ASCII case-insensitive. Duplicate `type` or `name` attributes, empty values, a language longer than 64 bytes, a name longer than 255 bytes, malformed or unmatched markers, nested openers, unclosed blocks, and more than 4,096 blocks reject the complete scan. Rejection produces no partial metadata; the compatibility renderer keeps the original source text so unsupported input cannot disappear or be reinterpreted as trusted output. Marker-like text inside a code body remains content unless the whole trimmed line is an exact case-insensitive closing marker.

## Evidence and tests

The focused tests in `deepwell/src/services/render/compat_fallback_code.rs` cover multiple blocks, mixed HTML and code source ordering, uppercase attributes, marker text inside a body, whitespace after a closer, malformed and unbalanced markers, nested openers, duplicate and invalid metadata, overlong names, and the exact block-count boundary. Existing compatibility-renderer tests cover emitted HTML, literal preservation of an unclosed block, collapsible composition, and preservation of hosted code metadata. The focused scanner suite passed with five tests on 2026-07-15.

## FTML backlog decision

This remains existing BND-01 debt and does not establish a second supported Wikidot grammar in Deepwell. Add a delayed FTML code structure that preserves raw source span, body, language, name, validity, and source order without requiring the surrounding document to render successfully. The Wikijump runtime should consume that structure to persist hosted code resources while FTML remains independent of page storage and routing.

## Migration condition

Remove `scan_compat_code_blocks` and the fallback-owned `CodeBlock` reconstruction when the pinned FTML version exposes the delayed structure above, malformed-input tests prove equivalent literal preservation, and the corpus plus browser gates prove that component-hosted CSS and JavaScript continue to resolve through `/local--code/`.

## Owner

Rokurolize.

## Review trigger

Re-evaluate on every FTML pin that changes code-block parsing or delayed syntax representation, and before removing or materially changing the BND-01 compatibility fallback.
