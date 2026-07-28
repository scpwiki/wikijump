# Block Quotes syntax

- Feature ID: `syntax-block-quotes`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented block quotes syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:block-quotes/source.wikidot.txt:1` through line 31 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:block-quotes (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:block-quotes/source.wikidot.txt:1` through line 31  
SHA-256 of complete source file: `9b53db6028afb8c260afc1878ccc0943dc56ff8a47b57d87d3077b0dfdf22589`

```wikidot
L0001 You can mark a blockquote by starting a line with one or more {{'>'}} characters, followed by a space and the text to be quoted.
L0002 
L0003 [[code]]
L0004 This is normal text here.
L0005 
L0006 > Indent me! The quick brown fox jumps over the lazy dog. \
L0007 Now this the time for all good men to come to the aid of \
L0008 their country. Notice how we can continue the block-quote \
L0009 in the same "paragraph" by using a backslash at the end of \
L0010 the line.
L0011 >
L0012 > Another block, leading to...
L0013 >> Second level of indenting. This second is indented even \
L0014 more than the previous one.
L0015 
L0016 Back to normal text.
L0017 [[/code]]
L0018 
L0019 This is normal text here.
L0020 
L0021 > Indent me! The quick brown fox jumps over the lazy dog. \
L0022 Now this the time for all good men to come to the aid of \
L0023 their country. Notice how we can continue the block-quote \
L0024 in the same "paragraph" by using a backslash at the end of \
L0025 the line.
L0026 >
L0027 > Another block, leading to...
L0028 >> Second level of indenting. This second is indented even \
L0029 more than the previous one.
L0030 
L0031 Back to normal text.
```
