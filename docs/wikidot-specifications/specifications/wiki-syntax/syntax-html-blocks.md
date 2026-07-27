# Html Blocks syntax

- Feature ID: `syntax-html-blocks`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented html blocks syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:html-blocks/source.wikidot.txt:1` through line 32 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:html-blocks (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:html-blocks/source.wikidot.txt:1` through line 32  
SHA-256 of complete source file: `7281ba038537ed0827daac9ef8edb1ec286df527fee6ea43e51c3c54be28fa09`

```wikidot
L0001 Create HTML blocks by using {{[[html]] ... @@[[/html]]@@}} tags (each on its own line). HTML block is a Code Block inserted in the IFRAME. It makes [http://community.wikidot.com/howto:use-html-scripting HTML - scripting] much easier.
L0002 
L0003 [[code type="html"]]
L0004 [[html]]
L0005 <h1>Custom HTML</h1>
L0006 <p>Something else</p>
L0007 <img src="anything.png" alt="hello ;-)"/>
L0008 [[/html]]
L0009 [[/code]]
L0010 
L0011 All wiki syntax inside a html block is treated as literal text and not processed.
L0012 
L0013 You can apply styles (both by means of <style type="text/css">...</style> and <element style="...">) to elements, but styling html and body (that are added transparently to your content if needed) is not supported. If you need any styling done to the top level elements, do this by wrapping the whole content of @@[[html]]@@ block in div with proper style, for example:
L0014 
L0015 [[code]]
L0016 [[html]]
L0017 <div style="background-color: black; color: lightgreen">
L0018 <p>This is a test.</p>
L0019 </div>
L0020 [[/html]]
L0021 [[/code]]
L0022 
L0023 This renders the whole HTML block black with text color light green:
L0024 
L0025 [[html]]
L0026 <div style="background-color: black; color: lightgreen">
L0027 <p>This is a test.</p>
L0028 </div>
L0029 [[/html]]
L0030 
L0031 Each HTML block on a page has a unique URL that lets you access it individually. You can do it by right clicking on the HTML block element on the rendered page and choose "Show only this frame" and check the web address.
L0032 ~~~~~~~~~~
```
