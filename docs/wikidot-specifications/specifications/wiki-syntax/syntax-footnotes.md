# Footnotes syntax

- Feature ID: `syntax-footnotes`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented footnotes syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:footnotes/source.wikidot.txt:1` through line 14 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:footnotes (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:footnotes/source.wikidot.txt:1` through line 14  
SHA-256 of complete source file: `ad0eaf24c936ff09371f4e9895074c091776551223ac1ac90b6f481a0363f0f8`

```wikidot
L0001 To make footnotes in the text use {{@@[[footnote]]@@}} block. To force the list of footnotes
L0002 to appear __not__ at the end of the page, use {{@@[[footnoteblock]]@@}}.
L0003 [[code]]
L0004 Some text[[footnote]]And a small footnote.[[/footnote]]. Here we go
L0005 with another footnote[[footnote]]Content of another footnote.[[/footnote]].
L0006 
L0007 [[footnoteblock]]
L0008 [[/code]]
L0009 
L0010 Some text[[footnote]]And a small footnote.[[/footnote]]. Here we go with another footnote[[footnote]]Content of another footnote.[[/footnote]].
L0011 
L0012 [[footnoteblock]]
L0013 
L0014 If you are not satisfied with the default title ("Footnotes") you can force your own title by using {{@@[[footnoteblock title="Custom title"]]@@}} or even do not use title at all (title="").
```
