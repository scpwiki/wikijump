# Headings syntax

- Feature ID: `syntax-headings`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented headings syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:headings/source.wikidot.txt:1` through line 10 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:headings (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:headings/source.wikidot.txt:1` through line 10  
SHA-256 of complete source file: `39275ab5243831a1cc36dba1fa0588988e3fca57acad34813966c51fde2187ea`

```wikidot
L0001 To make a heading start a line with a "plus". Make as many pluses as the heading level you want to get.
L0002 
L0003 [[code]]
L0004 + Level 1 Heading
L0005 ++ Level 2 Heading
L0006 +++ Level 3 Heading
L0007 ++++ Level 4 Heading
L0008 +++++ Level 5 Heading
L0009 ++++++ Level 6 Heading
L0010 [[/code]]
```
