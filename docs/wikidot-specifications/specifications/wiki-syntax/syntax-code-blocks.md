# Code Blocks syntax

- Feature ID: `syntax-code-blocks`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented code blocks syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:code-blocks/source.wikidot.txt:1` through line 55 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:code-blocks (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:code-blocks/source.wikidot.txt:1` through line 55  
SHA-256 of complete source file: `476292f230ba76a2ffc0fe7d4a81bb18ec172ab5f573454a7d0e7f830f064602`

```wikidot
L0001 Create code blocks by using {{[[code]]...[[/code]]}} tags (each on its own line).
L0002 
L0003 [[code]]
L0004 This is an example code block!
L0005 [[/code]]
L0006 
L0007 All wiki syntax inside a code block //except// @@[[include]]@@ tags is treated as literal text and not processed.  To prevent an include tag from being processed, put a single space in front of it.
L0008 
L0009 Each code block on a page has a unique URL that lets you access it individually.  This is especially useful for code blocks that contain CSS code (type = "css"):
L0010 
L0011 [[code]]
L0012 http://mysite.wikidot.com/category:page/code
L0013 http://mysite.wikidot.com/category:page/code/2  -- second block
L0014 [[/code]]
L0015 
L0016 This way you can extract code blocks defined in the page source itself, without taking any _template into account. To access code blocks form page source combined with _template, use the following URLs:
L0017 
L0018 [[code]]
L0019 http://mysite.wikidot.com/category:page/code_  -- note the trailing underscore
L0020 http://mysite.wikidot.com/category:page/code_/2  -- second block
L0021 [[/code]]
L0022 
L0023 To create PHP blocks that get automatically colorized when you use PHP tags, simply surround the code with {{[[code //type="php"//]]...[[/code]]}} tags).
L0024 
L0025 To get PHP code colorized you should surround it with <?php.. ?>.
L0026 
L0027 Wikidot.com uses PEAR::Text_Highlighter and supports a number of color schemes. Here is what is supported (allowed type values):
L0028 
L0029 php, html, cpp, css, diff, dtd, java, javascript, perl, python, ruby, xml.
L0030 
L0031 [[div style="float: left; width: 45%; margin: 0 2%;"]]
L0032 [[code]]
L0033 [[code type="php"]]
L0034 <?php
L0035 /* comment */
L0036 for($i=0; $i<100; $i++){
L0037 echo "number".$i."\n";
L0038 }
L0039 ?>
L0040 [[/code]]
L0041 [[/code]]
L0042 
L0043 [[/div]]
L0044 [[div style="float: left; width: 45%; margin: 0 2%;"]]
L0045 
L0046 [[code type="php"]]
L0047 <?php
L0048 /* comment */
L0049 for($i=0; $i<100; $i++){
L0050 echo "number".$i."\n";
L0051 }
L0052 ?>
L0053 [[/code]]
L0054 [[/div]]
L0055 ~~~~~~~~~~
```
