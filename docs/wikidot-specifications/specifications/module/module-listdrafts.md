# ListDrafts Module

- Feature ID: `module-listdrafts`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ListDrafts` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listdrafts-module/source.wikidot.txt:1` through line 9 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:listdrafts-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listdrafts-module/source.wikidot.txt:1` through line 9  
SHA-256 of complete source file: `e0020322623cbc39360e13bb88aa52c64fa079c513b3ec5c5f47c4b7fe723430`

```wikidot
L0001 This module lists all pages on Site where there is a draft included. You can choose if you want to display all draft or only for existing/non-existing pages.
L0002 
L0003 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0004 || pageType || no || exists, notexists || - || when not defined, all drafts are listed ||
L0005 
L0006 Example:
L0007 [[code]]
L0008 [[module ListDrafts pageType="exists"]]
L0009 [[/code]]
```
