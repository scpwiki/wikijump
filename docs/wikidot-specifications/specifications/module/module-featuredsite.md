# FeaturedSite Module

- Feature ID: `module-featuredsite`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `FeaturedSite` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:featuredsite-module/source.wikidot.txt:1` through line 19 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:featuredsite-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:featuredsite-module/source.wikidot.txt:1` through line 19  
SHA-256 of complete source file: `1b229e7c2c0528285beaab3820dca0bab36f229f67895f78e2e2c5561180c619`

```wikidot
L0001 ++ Description
L0002 
L0003 FeaturedSite module is very similar to SiteGrid, but it's displaying **only one** wiki site thumbnail and the thumbnail is much bigger.
L0004 
L0005 This module has no attributes.
L0006 
L0007 ++ Example
L0008 
L0009 [[code]]
L0010 [[module FeaturedSite]]
L0011 community.wikidot.com
L0012 [[/module]]
L0013 [[/code]]
L0014 
L0015 Which transfers to...
L0016 
L0017 [[module FeaturedSite]]
L0018 community.wikidot.com
L0019 [[/module]]
```
