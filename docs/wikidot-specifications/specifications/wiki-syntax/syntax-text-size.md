# Text Size syntax

- Feature ID: `syntax-text-size`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented text size syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:text-size/source.wikidot.txt:1` through line 25 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:text-size (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:text-size/source.wikidot.txt:1` through line 25  
SHA-256 of complete source file: `0aa6731250d9fe47c83de2998576ca40292af256975c2b41a4792178bfa9329a`

```wikidot
L0001 Text (font) size can be set with the {{@@[[size@@}} ...{{@@]]@@}} ... {{@@[[/size]]@@}} tags.  These tags can be nested.
L0002 
L0003 +++ Relative text sizes
L0004 Relative text sizes are based on the current font -- they increase or decrease the current font size.  To specify a relative text size use {{@@[[size smaller]]@@}}, {{@@[[size larger]]@@}}, {{@@[[size@@}} //n//{{@@em]]@@}}, or {{@@[[size@@}} //n//{{@@%]]@@}}, where //n// is a 1- to 5-digit number (including an optional decimal point).
L0005 ||~ what you type ||~ what you get ||
L0006 || {{@@[[size smaller]]smaller text[[/size]]@@}} || [[size smaller]]smaller text[[/size]] ||
L0007 || {{@@[[size larger]]larger text[[/size]]@@}} || [[size larger]]larger text[[/size]] ||
L0008 || {{@@[[size 80%]]80% of current size[[/size]]@@}} || [[size 80%]]80% of current size[[/size]] ||
L0009 || {{@@[[size 100%]]100% of current size[[/size]]@@}} || [[size 100%]]100% of current size[[/size]] ||
L0010 || {{@@[[size 150%]]150% of current size[[/size]]@@}} || [[size 150%]]150% of current size[[/size]] ||
L0011 || {{@@[[size 0.8em]]80% of current size[[/size]]@@}} || [[size 0.8em]]80% of current size[[/size]] ||
L0012 || {{@@[[size 1em]]100% of current size[[/size]]@@}} || [[size 1em]]100% of current size[[/size]] ||
L0013 || {{@@[[size 1.5em]]150% of current size[[/size]]@@}} || [[size 1.5em]]150% of current size[[/size]] ||
L0014 
L0015 +++ Absolute text sizes
L0016 Absolute text sizes are //not// based on the current font size.  To specify an absolute text size use {{@@[[size xx-small]]@@}}, {{@@[[size x-small]]@@}}, {{@@[[size small]]@@}}, {{@@[[size large]]@@}}, {{@@[[size x-large]]@@}}, {{@@[[size xx-large]]@@}}, or {{@@[[size@@}} //n//{{@@px]]@@}}, where //n// is a 1- to 5-digit number (including an optional decimal point).
L0017 ||~ what you type ||~ what you get ||
L0018 || {{@@[[size xx-small]]xx-small text[[/size]]@@}} || [[size xx-small]]xx-small text[[/size]] ||
L0019 || {{@@[[size x-small]]x-small text[[/size]]@@}} || [[size x-small]]x-small text[[/size]] ||
L0020 || {{@@[[size small]]small text[[/size]]@@}} || [[size small]]small text[[/size]] ||
L0021 || {{@@[[size large]]large text[[/size]]@@}} || [[size large]]large text[[/size]] ||
L0022 || {{@@[[size x-large]]x-large text[[/size]]@@}} || [[size x-large]]x-large text[[/size]] ||
L0023 || {{@@[[size xx-large]]xx-large text[[/size]]@@}} || [[size xx-large]]xx-large text[[/size]] ||
L0024 || {{@@[[size 7px]]text size 7 pixels[[/size]]@@}} || [[size 7px]]text size 7 pixels[[/size]] ||
L0025 || {{@@[[size 18.75px]]text size 18.75 pixels[[/size]]@@}} || [[size 18.75px]]text size 18.75 pixels[[/size]] ||
```
