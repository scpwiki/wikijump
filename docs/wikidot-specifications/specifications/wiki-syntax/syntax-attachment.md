# Attached files syntax

- Feature ID: `syntax-attachment`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented attached files syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:attachment/source.wikidot.txt:1` through line 6 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:attachment (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:attachment/source.wikidot.txt:1` through line 6  
SHA-256 of complete source file: `10974f61f5188c87fd6400abc96090478a23656aee2b1cefe246f1daa5b0e0c8`

```wikidot
L0001 ||~ what you type ||~ what it means ||
L0002 || {{@@[[@@file //filename// | //custom-text//@@]]@@}} || produces a link to a file attached to this page. _
L0003 //custom-text// changes the name of a link (//custom-text// will be displayed instead of the file name). ||
L0004 || {{@@[[@@file ///another-page/filename// | //custom-text//@@]]@@}} || produces a link to a file attached to //another-page// ||
L0005 
L0006 The destination file must be first attached to the page -- by clicking "files" and "upload file" from the options at the bottom of any page.  //Do not use a leading slash on filenames, unless you really mean that they are attached to the default (start) page//.
```
