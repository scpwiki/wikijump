# Standalone buttons for page options syntax

- Feature ID: `syntax-buttons`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented standalone buttons for page options syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:buttons/source.wikidot.txt:1` through line 24 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:buttons (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:buttons/source.wikidot.txt:1` through line 24  
SHA-256 of complete source file: `f6f7c04da59ec4a7136cdbf9a1a8551543a9a8f338653c0683c69886b76f820f`

```wikidot
L0001 Sometimes it might be convenient to hide the default page options and present only selected buttons to the users. The syntax for accomplishing this is:
L0002 
L0003 {{@@[[button@@ //type// //options//]]}}
L0004 
L0005 Where the //type// is: {{edit}}, {{edit-append}}, {{edit-sections}}, {{history}}, {{print}}, {{files}}, {{tags}}, {{source}} (view page source), {{backlinks}}, {{talk}} (works similar as in MediaWiki/Wikipedia), {{delete}}, {{rename}}, {{site-tools}}, {{edit-meta}}, {{watchers}}, {{parent}} and {{lock-page}}.
L0006 
L0007 Possible attributes are:
L0008 * text -- alternative text to be displayed
L0009 * class -- CSS class of the A element
L0010 * style -- CSS style definition
L0011 
L0012 For some nice "view source" and "print" buttons with icons you can use the following code:
L0013 [[code]]
L0014 [[>]]
L0015 [[button source style="background-image: url(http://www.wikidot.com/local--files/files/view-source.png); background-repeat: no-repeat; background-position: bottom right; padding-right: 20px; color: #444"]]
L0016 [[button print style="background-image: url(http://www.wikidot.com/local--files/files/document-print.png); background-repeat: no-repeat; background-position: bottom right; padding-right: 20px;color: #444"]]
L0017 [[/>]]
L0018 [[/code]]
L0019 
L0020 to get:
L0021 [[>]]
L0022 [[button source style="background-image: url(http://www.wikidot.com/local--files/files/view-source.png); background-repeat: no-repeat; background-position: bottom right; padding-right: 20px; color: #444"]]
L0023 [[button print style="background-image: url(http://www.wikidot.com/local--files/files/document-print.png); background-repeat: no-repeat; background-position: bottom right; padding-right: 20px;color: #444"]]
L0024 [[/>]]
```
