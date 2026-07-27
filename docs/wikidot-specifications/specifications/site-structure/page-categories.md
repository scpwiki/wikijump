# Page categories and namespaces

- Feature ID: `page-categories`
- Category: `site-structure`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot site-structure capability “Page categories and namespaces”, including its identity, relationships, routes, and rendering implications.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:35` through line 49 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:96` through line 100 (supporting)

## Documentation-derived behavioral evidence

### doc:site-structure (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:35` through line 49  
SHA-256 of complete source file: `20e91b5e74e135e07d4559a7057d2a43ce36b0e3db98fd3c8b20c10a5468b33f`

```wikidot
L0035 ++ Categories (namespaces)
L0036 
L0037 Although all the pages reside in the //flat structure//, pages can belong to different //categories// (//namespaces//). This allows:
L0038 * easier page management and structure,
L0039 * separate appearance settings, permissions, license for each category (see [[[doc:ManageSite module]]]),
L0040 * easier listing (see [[[doc:Pages module]]])
L0041 
L0042 Categories are uniquely identified by their //unix names//. Each page belongs to a certain category based on its //page unix name// which can have the form:
L0043 
L0044 = //category-unix-name//://the-rest//
L0045 
L0046 Everything that precedes the colon (':') in the //page unix name// is treated as a category name.
L0047 
L0048 Categories are created (when a page with a new category name is created) and automatically deleted (when no more pages contain category name).
L0049 
```

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:96` through line 100  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0096 +++ CATEGORIES
L0097 Categorize your content using page categories (namespaces), tags and "parent page" relation. Generate tag clouds, listings or even structured site maps.
L0098 
L0099 
L0100 
```
