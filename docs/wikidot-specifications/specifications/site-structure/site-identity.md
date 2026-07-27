# Sites and site identity

- Feature ID: `site-identity`
- Category: `site-structure`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot site-structure capability “Sites and site identity”, including its identity, relationships, routes, and rendering implications.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:5` through line 16 (canonical)

## Documentation-derived behavioral evidence

### doc:site-structure (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:5` through line 16  
SHA-256 of complete source file: `20e91b5e74e135e07d4559a7057d2a43ce36b0e3db98fd3c8b20c10a5468b33f`

```wikidot
L0005 + Sites
L0006 
L0007 Each Site powered by WikiDot.com resides in a unique subdomain of wikidot.com:
L0008 
L0009 = {{``http://``//site-unix-name//.wikidot.com}}
L0010 
L0011 where the {{site-unix-name}} consists of only alphanumeric characters (0..9, 'a'..'z') and ('-'). 
L0012 
L0013 Each site is independent from other sites but all the sites share User accounts, but can have different appearance, permission system, block-list etc.  In this context the wikidot.com service only provides hosting for the sites.
L0014 
L0015 From the functional point of view any site can consist of two (somehow overlapping) parts: content pages and forum section.
L0016 
```
