# Typography syntax

- Feature ID: `syntax-typography`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented typography syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:typography/source.wikidot.txt:1` through line 12 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:typography (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:typography/source.wikidot.txt:1` through line 12  
SHA-256 of complete source file: `b11324875781eec51d817767ce0e7bb6a01d64f0f6d869e1b4a66b6a66353393`

```wikidot
L0001 If you do care about typography there are a few ways to improve it in your text:
L0002 
L0003 ||~ you type ||~ you get||
L0004 || {{@@``quotation'' @@}} || ``quotation''||
L0005 || {{@@`quotation' @@}} || `quotation' ||
L0006 || {{@@,,quotation''@@}} || ,,quotation'' ||
L0007 || {{@<&lt;&lt;quotation&gt;&gt;>@}} || <<quotation>> ||
L0008 || {{@@>>quotation<<@@}} || >>quotation<< ||
L0009 || {{@@dots...@@}} || dots... ||
L0010 || {{@@em -- dash@@}} || em -- dash ||
L0011 
L0012 Note: em dash works only when surrounded by spaces.
```
