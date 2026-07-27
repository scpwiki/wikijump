# Collapsible Blocks syntax

- Feature ID: `syntax-collapsible-blocks`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented collapsible blocks syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:collapsible-blocks/source.wikidot.txt:1` through line 17 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:collapsible-blocks (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:collapsible-blocks/source.wikidot.txt:1` through line 17  
SHA-256 of complete source file: `94f1a86886c955c3bffddd47a2d3f97068c3214bd6e80dd166508f0fea22a5df`

```wikidot
L0001 [[collapsible show="+ explain collapsible blocks" hide="- ok, thanks"]]
L0002 The @@[[collapsible]]@@ tag lets you place a block of text on your page that the user can show/hide with one click.  For short blocks of text, use this form:
L0003 
L0004 [[code]]
L0005 [[collapsible show="+ Show whatever" hide="- Hide whatever"]]
L0006 Whatever text to show/hide.
L0007 [[/collapsible]]
L0008 [[/code]]
L0009 
L0010 You can format the collapsible block text as for any Wikidot text.  Look at the example at: http://snippets.wikidot.com/code:collapsible-block-unleashed.
L0011 
L0012 Put the @@[[collapsible]]@@ and @@[[/collapsible]]@@ tags on their own lines or the parser will not recognize them.
L0013 
L0014 By default, the show link says "+ show block" and the hide link says "- hide block".  For longer blocks, add the @@hideLocation="both"@@ option to show the hide link at the bottom as well as the top of the block when it's shown.  Other values for the hideLocation option are "top" (the default) and "bottom".
L0015 
L0016 Finally, you can use the @@folded="no"@@ option to show blocks by default, allowing the user to hide them if wanted.  We use this for tables of content, for example.
L0017 [[/collapsible]]
```
