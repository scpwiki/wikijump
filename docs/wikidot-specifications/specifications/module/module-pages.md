# Pages Module

- Feature ID: `module-pages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Pages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pages-module/source.wikidot.txt:1` through line 43 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:pages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pages-module/source.wikidot.txt:1` through line 43  
SHA-256 of complete source file: `4f68151642720d9ab4165dc2c5b38c040a0dd71c6f0bc26490ad9b071b82ed46`

```wikidot
L0001 **This module is deprecated.  Use the [/doc:listpages-module ListPages module] instead.**
L0002 
L0003 ++ Description
L0004 
L0005 The //Pages// module is able to list pages within a site or within a category.
L0006 
L0007 ++ Attributes
L0008 
L0009 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0010 || category || no || name of an existing category || none || limits the listing to the specified category||
L0011 || details || no || "true" || none || print extra information about pages ||
L0012 || preview || no || "true" || none || --prints a short preview of the page-- (This parameter is temporarily ignored due to performance reasons. It will come back soon) ||
L0013 || order || no || {{dateCreatedDesc}} _
L0014 {{dateCreatedAsc}} _
L0015 {{dateEditedDesc}} _
L0016 {{dateEditedAsc}} _
L0017 {{titleDesc}} _
L0018 {{titleAsc}} || {{titleAsc}} || selects ordering of the pages ||
L0019 || limit || no || any positive integer || none || how many pages to return; if you omit this all pages will be listed ||
L0020 
L0021 ++ Examples
L0022 
L0023 
L0024 
L0025 Print only pages from the //_default// category with details:
L0026 [[code]]
L0027 [[module Pages category="_default" details="true"]]
L0028 [[/code]]
L0029 
L0030 Print all pages with details and preview:
L0031 [[code]]
L0032 [[module Pages details="true" preview="true"]]
L0033 [[/code]]
L0034 
L0035 10 most recently edited pages
L0036 [[code]]
L0037 [[module Pages order="dateEditedDesc" limit="10"]]
L0038 [[/code]]
L0039 
L0040 10 most recently created pages
L0041 [[code]]
L0042 [[module Pages order="dateCreatedDesc" limit="10"]]
L0043 [[/code]]
```
