# ChildPages Module

- Feature ID: `module-childpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ChildPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### ChildPages lists current children with live DOM, empty-state, and unknown-argument behavior

- Observation ID: `childpages-live-list-empty-and-unknown-arguments`
- Classification: `documentation-clarification`
- Observed at: `2026-07-28`
- Analysis: The ChildPages documentation states only that the deprecated module lists children of the containing page in alphabetical order and has no required attributes. Controlled run-owned sandbox pages show the exact output shape and several undocumented boundaries: live Wikidot wraps non-empty output in div.child-pages-block containing a ul of linked li rows, includes child pages from categories other than the parent page's category, includes underscore-prefixed hidden pages when they are viewable, emits no wrapper at all for an empty child set, and ignores unknown attributes while still rendering the module.

Normative behavior:

- ChildPages selects pages whose parent relation points to the page containing the module.
- ChildPages is not restricted to the containing page's category; child pages from other categories are included when viewable.
- ChildPages includes underscore-prefixed hidden child pages when the anonymous viewer may view them.
- ChildPages rows are ordered alphabetically by title using live Wikidot's case-insensitive title order.
- A non-empty ChildPages render emits div.child-pages-block containing ul and one li anchor per child page.
- An empty ChildPages render emits no wrapper, list, row, or literal module text.
- ChildPages ignores unknown attributes while rendering children.

Evidence:

- `install/local/wikidot-verification/artifacts/childpages-module-live.json` (SHA-256 `3362d5122487becb7a48cea1291dc4745c1c48919de48235f3f687d73669a588`), cases: `bare-childpages-with-cross-category-and-hidden-children`, `empty-childpages`, `unknown-argument-with-children`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:childpages-module/source.wikidot.txt:1` through line 15 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:childpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:childpages-module/source.wikidot.txt:1` through line 15  
SHA-256 of complete source file: `dfb8962f842f60c17793a5e4521cc39e22a78fb70723a649f7b77f81a4282256`

```wikidot
L0001 **This module is deprecated.  Use the [/doc:listpages-module ListPages module] with the parent selector instead.**
L0002 
L0003 ++ Description
L0004 
L0005 Lists children pages of the page that contains the module. The list is ordered alphabetically.
L0006 
L0007 ++ Attributes
L0008 
L0009 No attributes required.
L0010 
L0011 ++ Examples
L0012 
L0013 [[code]]
L0014 [[module ChildPages]]
L0015 [[/code]]
```
