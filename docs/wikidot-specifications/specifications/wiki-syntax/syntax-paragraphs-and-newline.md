# Paragraphs and newlines syntax

- Feature ID: `syntax-paragraphs-and-newline`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented paragraphs and newlines syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:paragraphs-and-newline/source.wikidot.txt:1` through line 55 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:paragraphs-and-newline (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:paragraphs-and-newline/source.wikidot.txt:1` through line 55  
SHA-256 of complete source file: `d499bebc90ea5b2ca96da61f74105b5ee6bac527ddde22d6798a615c4dc590f2`

```wikidot
L0001 Paragraphs are separated by two new lines. One new line produces a... new line.
L0002 
L0003 [[code]]
L0004 First paragraph. Lorem ipsum dolor sit amet, consectetuer adipiscing elit.
L0005 
L0006 Second paragraph. Aenean a libero. Vestibulum adipiscing, felis ac faucibus imperdiet, erat lacus accumsan neque, vitae nonummy lorem pede ac elit.
L0007 Just a new line.
L0008 Another new line.
L0009 [[/code]]
L0010 
L0011 First paragraph. Lorem ipsum dolor sit amet, consectetuer adipiscing elit.
L0012 
L0013 Second paragraph. Aenean a libero. Vestibulum adipiscing, felis ac faucibus imperdiet, erat lacus accumsan neque, vitae nonummy lorem pede ac elit.
L0014 Just a new line.
L0015 Another new line.
L0016 
L0017 Line-Break:
L0018 there is a special character used for line-break at the end of a line which does not start a new paragraph and is very useful in tables or divs where it is sometimes needed:
L0019 it contains a "space" and "underscore" (and then "enter" ) to insert such "\n" in the code and start at the next line.
L0020 
L0021 The difference is not easy to show ( because spaces + enter are rendered also in Code blocks to nearly one line):
L0022 
L0023 [[code]]
L0024 Only 5x space+ Enter:
L0025 
L0026 
L0027 
L0028 
L0029 
L0030 Last line after 5 x enter 
L0031 
L0032 With special Line-Break: _
L0033  _
L0034  _
L0035  _
L0036  _
L0037 Last line after 5 x special line-Break
L0038 [[/code]]
L0039 Only 5x space+ Enter:
L0040 
L0041 
L0042 
L0043 
L0044 
L0045 Last line after 5 x enter 
L0046 
L0047 With special Line-Break: _
L0048  _
L0049  _
L0050  _
L0051  _
L0052 Last line after 5 x special line-Break
L0053 
L0054 
L0055 Do not forget that spaces alone or empty lines are rendered to one line!
```
