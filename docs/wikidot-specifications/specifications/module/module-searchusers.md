# SearchUsers Module

- Feature ID: `module-searchusers`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `SearchUsers` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:searchusers-module/source.wikidot.txt:1` through line 23 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:searchusers-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:searchusers-module/source.wikidot.txt:1` through line 23  
SHA-256 of complete source file: `188cc7c3dd48164ce61f4d3e953da69d50c03dab2697abf419e68d96bccfddad`

```wikidot
L0001 ++ Description
L0002 
L0003 The SearchUsers module lets you search all Wikidot users by login id, email address, or full name.  You can place the SearchUsers module itself on any page, but your site //must// contain a page called "search:users" that (also) contains the SearchUsers module.
L0004 
L0005 ++ Attributes
L0006 
L0007 The SearchUsers module does not allow any attributes.
L0008 
L0009 ++ Example
L0010 
L0011 On your site start page:
L0012 
L0013 [[code]]
L0014 ++ Search all Wikidot users
L0015 [[module SearchUsers]]
L0016 = (enter email address or nick or real name)
L0017 [[/code]]
L0018 
L0019 On your site's search:users page:
L0020 
L0021 [[code]]
L0022 [[module SearchUsers]]
L0023 [[/code]]
```
