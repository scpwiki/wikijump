# Watchers Module

- Feature ID: `module-watchers`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Watchers` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:watchers-module/source.wikidot.txt:1` through line 20 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:watchers-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:watchers-module/source.wikidot.txt:1` through line 20  
SHA-256 of complete source file: `fdaa56cf10e58d87b6ce9c167dc2fe40528cfed3a9773033578926c0eec1ca95`

```wikidot
L0001 ++ Description
L0002 
L0003 This module is used to list users watching the page. It will display all users who will receive notification about the changes on this page, so: page watchers, category watchers and site watchers. 
L0004 
L0005 Module Watchers also displays "watch/unwatch" options for users.
L0006 
L0007 ++ Attributes
L0008 
L0009 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0010 || noActions || no || "true" || none || does not show "watch/unwatch" options ||
L0011 
L0012 ++ Examples
L0013 
L0014 List all users watching this page:
L0015 
L0016 [[code]]
L0017 [[module Watchers]]
L0018 [[/code]]
L0019 
L0020 [[module Watchers]]
```
