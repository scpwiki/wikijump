# SearchAll Module

- Feature ID: `module-searchall`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `SearchAll` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:searchall-module/source.wikidot.txt:1` through line 23 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:searchall-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:searchall-module/source.wikidot.txt:1` through line 23  
SHA-256 of complete source file: `09cf4af605644948e4b52a8ea7cf64b0933d6616390d8c731bed59334148bf7e`

```wikidot
L0001 ++ Description
L0002 
L0003 The SearchAll module lets your users search all Wikidot sites, including private sites (if they are logged in, and for private sites that they are members of).  You can place the SearchAll module itself on any page, but your site //must// contain a page called "search:all" that (also) contains the SearchAll module.
L0004 
L0005 ++ Attributes
L0006 
L0007 The SearchAll module does not allow any attributes.
L0008 
L0009 ++ Example
L0010 
L0011 On your site start page:
L0012 
L0013 [[code]]
L0014 ++ Search all Wikidot sites
L0015 [[module SearchAll]]
L0016 = ([http://www.wikidot.com/doc:searching Search tips])
L0017 [[/code]]
L0018 
L0019 On your site's search:all page:
L0020 
L0021 [[code]]
L0022 [[module SearchAll]]
L0023 [[/code]]
```
