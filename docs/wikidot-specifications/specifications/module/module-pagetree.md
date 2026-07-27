# PageTree Module

- Feature ID: `module-pagetree`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `PageTree` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagetree-module/source.wikidot.txt:1` through line 22 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:pagetree-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagetree-module/source.wikidot.txt:1` through line 22  
SHA-256 of complete source file: `4933ecb99575fa43ea00d880fddca42d2621e3c145eed71a5f884e22cd693c4d`

```wikidot
L0001 ++ Description
L0002 
L0003 This module can visualize the structure of pages connected by //parenthood// - i.e. the page tree is constructed by the fact that a page can have a //parent page//. This can be set by accessing page //+options// and clicking on //parent//.
L0004 
L0005 Parenthood also affects navigation since it produces a breadcrumb navigation element at the top of the page which is quite useful.
L0006 
L0007 ++ Attributes
L0008 
L0009 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0010 || root || no || page name string || current page || top element of the tree ||
L0011 || showRoot || no || {{"true"}} || {{"false"}} || should the root element be displayed on the top of the list? ||
L0012 || depth || no || integer {{n > 0}} || none || limits maximum depth of the list; {{n = "1"}} displays only child pages of the root page, {{n = "2"}} displays child pages and their child pages etc. ||
L0013 
L0014 ++ Examples
L0015 
L0016 Display page tree of the documentation:
L0017 
L0018 [[code]]
L0019 [[module PageTree root="doc" showRoot="true"]]
L0020 [[/code]]
L0021 
L0022 [[module PageTree root="doc" showRoot="true"]]
```
