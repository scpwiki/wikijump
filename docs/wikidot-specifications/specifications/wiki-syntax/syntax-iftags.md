# Iftags syntax

- Feature ID: `syntax-iftags`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented iftags syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:iftags/source.wikidot.txt:1` through line 27 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:iftags (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:iftags/source.wikidot.txt:1` through line 27  
SHA-256 of complete source file: `94a65c2ac891be332f870ed6b3f96ed0b60a05c18f6622e32a9427fd6cb7febb`

```wikidot
L0001 Tags are kind of special labels for a page, manually added in by editors by clicking on the **tags** link at the page options buttons on bottom of a page. Every tag can be max 64 characters long, tags are "space" separated and there is no limit of tags per page. Tags are very useful to label pages and then it's easy to create Tag Cloud, which allow to find interesting pages or topics much faster.
L0002 
L0003 Special tags start with an underline: they are not automatically shown in tag clouds, but they can be used as special limitations in [[iftag]] conditions. Tags can be used in ListPages Module with generic conditions ( +, - ) too.
L0004 
L0005 **Iftag** is a special condition question. You can use it on every page to "react" on tags and set up on the particular page used .
L0006 
L0007 Syntax:
L0008 [[code]]
L0009 [[iftags +tag1 -tag2 tag3]] ... [[/iftags]]
L0010 [[/code]]
L0011 where the +/-"tag#" stands for the requested tag-indexes.
L0012 * + before a tagname means - this tag must exist, (tag without a modifier works in a same way)
L0013 * - before a tagname means - this tag must not exist.
L0014 
L0015 Example:
L0016 [[code]]
L0017 [[iftags +science]]
L0018 This page is labeled as: science.
L0019 
L0020 Click here to view more science articles >
L0021 [[/iftags]]
L0022 
L0023 [[iftags +bug -fixed]]
L0024 This is a bug, but it's not fixed yet.
L0025 [[/iftags]]
L0026 
L0027 [[/code]]
```
