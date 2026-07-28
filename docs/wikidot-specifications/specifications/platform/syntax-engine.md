# Wiki syntax engine

- Feature ID: `syntax-engine`
- Category: `platform`
- Documentation status: `high-level-documentation`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot capability “Wiki syntax engine” and its user-visible configuration, state, permissions, and output.

## Implementation contract

- The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.
- Account, site, category, page, and actor context MUST be enforced at the public service boundary.
- Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- The corpus describes this capability at product level. Use live Wikidot evidence to resolve any implementation detail the snapshot does not define.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:start/source.wikidot.txt:1` through line 11 (supporting)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:48` through line 52 (supporting)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:start (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:start/source.wikidot.txt:1` through line 11  
SHA-256 of complete source file: `80587bbfa3f1180473c6d12e39217ad1b709633b022e6f5eb7d0da89b3db8427`

```wikidot
L0001 This document describes the **Wiki Syntax** used within the Wikidot.com project.
L0002 
L0003 Any page of any of the sites exists in two different forms: the source code and the compiled code. Source code is  what you can edit and what describes the content of the page. Source code is compiled into the (XHTML) code that is sent to the browser when you view/browse the page. The Wiki Syntax is used to create content of the pages by editing the source code.
L0004 
L0005 If you are looking for wiki code snippets ready to copy/paste/modify, please visit our [http://snippets.wikidot.com Code Snippets Site].
L0006 
L0007 [[div style="border: 1px solid #BBB; padding: 5px 20px; background-color: #FFF;"]]
L0008 Documents you might also be interested in:
L0009 * [[[doc:embedding | Code embedding]]] -- list of supported embeds, i.e. pieces of code from other websites you can use on Wikidot, like films from YouTube or Google Gadgets.
L0010 * [[[doc:modules | Modules]]] -- description of //modules// -- interactive elements you can put on your pages.
L0011 [[/div]]
```

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:48` through line 52  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0048 +++ POWERFUL WIKI SYNTAX AND ENGINE
L0049 Our Wiki Syntax is certainly one of the most powerful available. [http://www.wikidot.com/doc:wiki-syntax Read more] or [http://sandbox.wikidot.com try it in the Sandbox] demo. Our engine is built not only to handle simple sites, but whole portals. Wikidot It also allows to embed LaTeX-style [*http://www.wikidot.com/doc:wiki-syntax#toc27 equations], [*http://www.wikidot.com/doc:wiki-syntax#toc30 bibliography items], [*http://www.wikidot.com/doc:wiki-syntax#toc29 footnotes] and more features. (also look at [*http://snippets.wikidot.com snippets.wikidot.com] for more examples)
L0050 
L0051 
L0052 
```
