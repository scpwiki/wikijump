# Table Of Contents syntax

- Feature ID: `syntax-table-of-contents`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented table of contents syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:table-of-contents/source.wikidot.txt:1` through line 24 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:table-of-contents (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:table-of-contents/source.wikidot.txt:1` through line 24  
SHA-256 of complete source file: `f79e8bca655717918d00aba2ec625caa3951ce8f968ba21479b350af8b9ef328`

```wikidot
L0001 To create a list of every heading, with a link to that heading, put a table of contents tag on its own line.
L0002 
L0003 [[code]]
L0004 [[toc]]
L0005 [[f>toc]] - right-float table of contents
L0006 [[f<toc]] - left-float table of contents
L0007 [[/code]]
L0008 
L0009 Note that the table of contents creates a bookmark called "#toc".  
L0010 
L0011 If you want a particular heading NOT to appear in the table of contents, append the pluses with an asterisk, like this:
L0012 
L0013 [[code]]
L0014 + This section appears in the TOC
L0015 +* And this one does not
L0016 ++* Neither does this one
L0017 [[/code]]
L0018 **TOC example using above code**
L0019 ----
L0020 [[toc]]
L0021 ----
L0022 + This section appears in the TOC
L0023 +* And this one does not
L0024 ++* Neither does this one
```
