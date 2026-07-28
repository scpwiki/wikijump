# OrphanedPages Module

- Feature ID: `module-orphanedpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `OrphanedPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:orphanedpages-module/source.wikidot.txt:1` through line 8 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:orphanedpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:orphanedpages-module/source.wikidot.txt:1` through line 8  
SHA-256 of complete source file: `309a49a5464f3cb606d56a7def9037395034bd8a2a1fe6d92a704dd2a453a670`

```wikidot
L0001 This module shows the list of pages that do not have any incoming links from other pages - at least internal links produced by syntax {{@@[[[page-name]]]@@}}. If a page is listed here in this module should not mean anything wrong because there might be special pages that do not (and should not as e.g. some forum pages) have incoming links. But it is recommended to check this list from time to time.
L0002 
L0003 The same module can be found in **site tools** at the bottom of every page.
L0004 
L0005 +++ Usage
L0006 [[code]]
L0007 [[module OrphanedPages]]
L0008 [[/code]]
```
