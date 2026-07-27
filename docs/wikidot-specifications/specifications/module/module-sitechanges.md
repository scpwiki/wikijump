# SiteChanges Module

- Feature ID: `module-sitechanges`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `SiteChanges` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:sitechanges-module/source.wikidot.txt:1` through line 24 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:sitechanges-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:sitechanges-module/source.wikidot.txt:1` through line 24  
SHA-256 of complete source file: `39e9752d03f6fb4f8e42e7ea32b7b09a2fc57a02c544319f39d023bea3aeb512`

```wikidot
L0001 ++ Description
L0002 
L0003 Lists recent changes made to the site content. Supported types of changes:
L0004 * new pages
L0005 * page content changes (edit source/title)
L0006 * page rename/move
L0007 * metadata change (parent page)
L0008 * attached file actions
L0009 
L0010 Changes do not include any forum activity. Use [[[doc:recentposts-module | RecentPosts]]] module for this.
L0011 
L0012 The module shows the category name for pages not in default category
L0013 
L0014 ++ Attributes
L0015 
L0016 No attributes required
L0017 
L0018 ++ Examples
L0019 
L0020 [[code]]
L0021 [[module SiteChanges]]
L0022 [[/code]]
L0023 
L0024 [[module SiteChanges]]
```
