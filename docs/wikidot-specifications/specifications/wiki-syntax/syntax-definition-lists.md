# Definition Lists syntax

- Feature ID: `syntax-definition-lists`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented definition lists syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:definition-lists/source.wikidot.txt:1` through line 21 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:definition-lists (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:definition-lists/source.wikidot.txt:1` through line 21  
SHA-256 of complete source file: `43ba8ffbe68eba94ef66c6f29230cc30821fc0899ec1733d81cac85c282ad1df`

```wikidot
L0001 You can create a definition (description) list with the following syntax:
L0002 
L0003 [[code]]
L0004 : Item 1 : Something
L0005 : Item 2 : Something else
L0006 [[/code]]
L0007 
L0008 : Item 1 : Something
L0009 : Item 2 : Something else
L0010 
L0011 If you need to put more than one line in the definition list, please use _ (underscore) at the end of the line you want to break (after one space). Remember not to insert any character after the underscore.
L0012 
L0013 [[code]]
L0014 : Item 1 : Something _
L0015 another line
L0016 : Item 2 : Something else
L0017 [[/code]]
L0018 
L0019 : Item 1 : Something _
L0020 another line
L0021 : Item 2 : Something else
```
