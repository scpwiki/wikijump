# PagesByTag Module

- Feature ID: `module-pagesbytag`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `PagesByTag` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagesbytag-module/source.wikidot.txt:1` through line 69 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:pagesbytag-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagesbytag-module/source.wikidot.txt:1` through line 69  
SHA-256 of complete source file: `994c0fec79ddd073ac82e5a85dd0dae91c4c532fb7bce57a18acb8485a3f6bad`

```wikidot
L0001 **This module is deprecated.  Use the [/doc:listpages-module ListPages module] with the tag and category selectors instead.  Specifically:**
L0002 
L0003 [[code]]
L0004 [[module ListPages tags="@URL" OTHER PARAMETERS]]
L0005 MODULE BODY
L0006 [[/module]]
L0007 [[/code]]
L0008 
L0009 ++ Description
L0010 
L0011 This module lists all pages, that are tagged[[footnote]]You can tag a page by clicking on the ``tags'' button at the bottom of a page[[/footnote]] with a specific tag.  The scope can optionally be restricted to a single category.
L0012 
L0013 ++ Attributes
L0014 
L0015 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0016 || {{tag}} || no || an all-lowercase text string || none || limits displayed pages to the specified tag ||
L0017 || {{category}} || no || valid category name || none || limits displayed pages to the specified page category ||
L0018 
L0019 ++ Notes
L0020 
L0021 If you do not specify a {{tag}} attribute, //PagesByTag//
L0022 * will use a tag that is specified by adding a trailing {{/tag///any-tagl//}} to the URL, like {{http:@@//@@//{wiki-name}//.wikidot.com@@/@@//{page-name}//@@/@@tag@@/@@//{any-tag}//}}
L0023 * will displays nothing if no tag is specified in the URL
L0024 
L0025 If you do not specify a {{category}} attribute, //PagesByTag//
L0026 * will use a category that is specified by adding a trailing {{/category///any-category//}} to the URL, like {{http:@@//@@//{wiki-name}//.wikidot.com@@/@@//{page-name}//@@/@@category@@/@@//{any-category}//}}
L0027 * will displays pages from all categories
L0028 
L0029 If //PagesByTag// is specified without attributes (i. e. @@[[module PagesByTag]]@@), it works nicely together with @@[[@@[http://www.wikidot.com/doc:tagcloud-module module TagCloud]@@]]@@.  If correctly set up (like on a default [[[system:page-tags|system:page-tags]]] page), //TagCloud// generates links of the form ...{{/tag///any-tag//}}, which //PagesByTag// then uses to list those pages.
L0030 
L0031 ++ [[# examples]]Examples
L0032 
L0033 ++++ PagesByTag Standalone
L0034 [[div style="float:right; width:50%; padding:0 1em"]]
L0035 //what you get ...//
L0036 [[module PagesByTag]]
L0037 [[size smaller]]If you don't see any //PagesByTag// output here, try adding {{/tag///any-tag//}} to the end of the ...{{/doc:pagesbytag-module}} URL. An example would be [http://www.wikidot.com/doc:pagesbytag-module/tag/news#examples doc:pagesbytag-module/tag/news][[/size]]
L0038 [[/div]]
L0039 //what you type ...//
L0040 [[code]]
L0041 [[module PagesByTag]]
L0042 [[/code]]
L0043 ~~~~>
L0044 
L0045 ++++ PagesByTag with Tag Attribute
L0046 [[div style="float:right; width:50%; padding:0 1em"]]
L0047 //what you get ...//
L0048 [[module PagesByTag tag="news"]]
L0049 [[/div]]
L0050 //what you type ...//
L0051 [[code]]
L0052 [[module PagesByTag tag="news"]]
L0053 [[/code]]
L0054 ~~~~>
L0055 
L0056 ++++ PagesByTag with Tag and Category Attribute
L0057 [[div style="float:right; width:50%; padding:0 1em"]]
L0058 //what you get ...//
L0059 [[module PagesByTag tag="news" category="_default"]]
L0060 [[/div]]
L0061 //what you type ...//
L0062 [[code]]
L0063 [[module PagesByTag tag="news" category="_default"]]
L0064 [[/code]]
L0065 ~~~~>
L0066 
L0067 ++ Credits
L0068 
L0069 The original text is located at the Wikidot Open Source Edition [http://www.wikidot.org/doc:pagesbytag-module documentation pages].
```
