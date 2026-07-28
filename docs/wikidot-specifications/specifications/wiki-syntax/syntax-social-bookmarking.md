# Social Bookmarking syntax

- Feature ID: `syntax-social-bookmarking`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented social bookmarking syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:social-bookmarking/source.wikidot.txt:1` through line 19 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:social-bookmarking (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:social-bookmarking/source.wikidot.txt:1` through line 19  
SHA-256 of complete source file: `c2dd86bda79accc9ddbe4b8403cc1e896346ee8dedb4b1203494ae21659a4343`

```wikidot
L0001 It is easy to add "social bookmarking" buttons to your pages -- just write {{@@[[social]]@@}} (without any parameters) and get:
L0002 
L0003 [[social blinklist,blogmarks,connotea,del.icio.us,digg,fark,feedmelinks,furl,linkagogo,newsvine,netvouz,reddit,simpy,spurl,wists,yahoomyweb,facebook]]
L0004 
L0005 This is equivalent to:
L0006 
L0007 [[code]]
L0008 [[social blinklist,blogmarks,connotea,del.icio.us,digg,fark,feedmelinks,furl,linkagogo,newsvine,netvouz,reddit,simpy,spurl,wists,yahoomyweb,facebook]]
L0009 [[/code]]
L0010 
L0011 You can also choose only selected services, e.g. to show digg, furl, del.icio.us and Facebook use:
L0012 
L0013 [[code]]
L0014 [[social digg,furl,del.icio.us,facebook]]
L0015 [[/code]]
L0016 
L0017 and get: [[social digg,furl,del.icio.us,facebook]]
L0018 
L0019 **Tip:** Use social bookmarking! It is always a good idea to put social shortcuts under an article or inside your side bar.
```
