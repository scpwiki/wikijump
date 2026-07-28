# Include syntax

- Feature ID: `syntax-include`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented include syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:include/source.wikidot.txt:1` through line 36 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:include (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:include/source.wikidot.txt:1` through line 36  
SHA-256 of complete source file: `0e58410affcb9cac566c11fc1121bb6ea2f140a00e8d667521034f9ab549e8fa`

```wikidot
L0001 If you want to include contents of another page use:
L0002 [[code]][[include pagename]]
L0003 [[/code]]
L0004 
L0005 or
L0006 
L0007 [[code]][[include :sitename:pagename]]
L0008 [[/code]]
L0009 
L0010 The //include// tag should start and end with a newline.  @@[[include]]@@ tags are parsed //inside// code blocks.  To prevent an @@[[include]]@@ tag from being parsed, put a space in front of it.  This does make copy/paste of example code that contains @@[[include]]@@ tags a problem.
L0011 
L0012 The sitename can be a Wikidot subdomain (e.g. :www) or a full name, including a custom domain.
L0013 
L0014 The {{[[include]]}} tag can also take parameters and substitute variables in the included source. To denote variables in the included page use:
L0015 
L0016 [[code]]
L0017 {$var1}, {$number_books}, {$title}, {$variable_name}, {$variableName}
L0018 [[/code]]
L0019 
L0020 and in the including page use:
L0021 
L0022 [[code]]
L0023  [[include pagename
L0024 |var1=value1
L0025 |number_books=43
L0026 |title=Best Wiki Ever
L0027 |variable_name=just a variable
L0028 |variableName=another variable
L0029 ]]
L0030 [[/code]]
L0031 
L0032 As you can see you can split variable definitions over several lines for cleaner code.
L0033 
L0034 **NOTE: includes and images/files**: The {{[[include]]}} works just by inserting the page source at a given place. If you have any images or files attached in the included page and you refer to them as @@[[image filename.jpg]]@@ in the included page, please rather use the image/file source with the name of the page too, e.g. @@[[image @@**included-page/**filename.jpg]]
L0035 
L0036 Includes across sites are called //cross-site includes// or CSIs.  CSIs are a powerful way to link page templates and code from other sites.
```
