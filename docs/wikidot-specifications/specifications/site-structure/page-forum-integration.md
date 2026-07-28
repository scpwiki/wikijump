# Page and forum integration

- Feature ID: `page-forum-integration`
- Category: `site-structure`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot site-structure capability “Page and forum integration”, including its identity, relationships, routes, and rendering implications.

## Implementation contract

- The persistence model MUST represent the documented entity and relationships.
- Public links, routes, selection behavior, permissions, and rendered structure MUST preserve those relationships.
- Imported Wikidot identifiers and URLs MUST remain compatibility-stable.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:119` through line 127 (canonical)

## Documentation-derived behavioral evidence

### doc:site-structure (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:119` through line 127  
SHA-256 of complete source file: `20e91b5e74e135e07d4559a7057d2a43ce36b0e3db98fd3c8b20c10a5468b33f`

```wikidot
L0119 + Interaction of Pages and Forum
L0120 
L0121 Forum infrastructure can be used to discuss pages (by creating a relation between apage and a forum thread) or even to add forum elements to content pages.
L0122 
L0123 To enable the "discuss" button at the bottom of a page the [[[doc:ManageSite module]]] should be use and option //Forum & discussion// -> //Per page discussion//.
L0124 
L0125 When enabled, a button "discuss" will be visible at the bottom page options bar which will lead to a unique forum thread just for commenting this particular page.
L0126 
L0127 Another way is to use the [[[doc:Comments module]]] and embed it in the page. This will bring the whole discussion just below the page content. Such a solution could  be used e.g. when one creates an article and wants other people to discuss it or comment.
```
