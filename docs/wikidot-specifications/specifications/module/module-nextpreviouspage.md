# NextPreviousPage Module

- Feature ID: `module-nextpreviouspage`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `NextPage` and `PreviousPage` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### NextPage and PreviousPage use ListPages output with live selection and a title-mode PreviousPage quirk

- Observation ID: `nextpreviouspage-live-selection-template-and-title-quirk`
- Classification: `documentation-correction`
- Observed at: `2026-07-28`
- Analysis: The NextPreviousPage documentation says the modules create links to adjacent pages in a category and use the same item format as ListPages, but it does not define the exact wrapper, empty output, malformed-argument behavior, or the production title-mode edge cases. Controlled run-owned sandbox pages show that both modules render through the ListPages wrapper and item template. The omitted by argument and by="date" use creation-date adjacency. NextPage by="title" selects the next greater title. PreviousPage by="title" has a live legacy quirk: in a five-page title matrix it returned the current page itself for every page, including the first and last rows, rather than the preceding title. Unknown attributes are ignored, an empty or malformed by attribute falls back to date mode, tags filter candidate pages while the current page remains the position anchor, and an empty result renders an empty list-pages-box.

Normative behavior:

- NextPage and PreviousPage render the selected row using ListPages' div.list-pages-box and div.list-pages-item containers.
- The module body is compiled as a ListPages item template; an empty body uses the default ListPages row template.
- When no row matches the requested neighbor, live Wikidot emits an empty div.list-pages-box rather than preserving the module literally.
- The category argument restricts candidate pages to comma- or whitespace-separated categories; category="*" searches all categories.
- With category omitted, candidates come from the current page category.
- The omitted by argument, by="date", by="", and a malformed by token use creation-date ordering.
- Date-mode PreviousPage selects the nearest visible candidate created before the current page; date-mode NextPage selects the nearest visible candidate created after it.
- Title-mode NextPage selects the first visible candidate whose title sorts after the current page title.
- Title-mode PreviousPage uses live Wikidot's inclusive title-mode behavior and returns the first visible candidate whose title sorts at or after the current page title; when the current page matches the selector this is the current page itself.
- The tags argument uses ListPages-style tag filtering: +tag requires a tag, -tag excludes a tag, unmodified tags match any listed tag, @URL reads the URL tag value, and = adds the current page's visible tags as any-tag candidates.
- Tag and category filters apply to candidate pages, but the current page's own title or creation date remains the position anchor even if the current page is not itself a candidate.
- Unknown attributes are ignored while recognized attributes from the same module invocation still apply.
- Pages hidden from the anonymous viewer are excluded from candidates.

Evidence:

- `install/local/wikidot-verification/artifacts/nextpreviouspage-module-live.json` (SHA-256 `1b2f88a11ef6b7af1b30fee98b72a1b3d87cd0a541c2fb86041ecf4e3563fe1d`), cases: `middle-documented-body-category-title-date-tags`, `first-title-boundary-and-empty-body`, `last-title-boundary-and-malformed-by`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:nextpreviouspage-module/source.wikidot.txt:1` through line 81 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:nextpreviouspage-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:nextpreviouspage-module/source.wikidot.txt:1` through line 81  
SHA-256 of complete source file: `6aa07837cc3b0a57ad4c35fcc4903b936c828f2e5b738078244a0010f65fc8ef`

```wikidot
L0001 **This module is deprecated.  Use the [/doc:listpages-module ListPages module] with the tag and category selectors instead.**
L0002 
L0003 The NextPage and PreviousPage modules automatically create links to the next or previous page in the category with several type of sorting.
L0004 
L0005 ++ Attributes 
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || category || no || comma- or space-separated names, * for all categories || current category || the category where next / previous pages are chosen from ||
L0009 || by || no || title, date || date || changes the way of choosing next / previous page ||
L0010 || tags || no || comma- or space-separated tag names with {{+}} and {{-}} modifiers, _
L0011 or {{@URL}} || none || lists tags that are used as a criteria to select pages, the "+" before the tag makes it required, "-" means "without a tag" and tags without modifiers translate to "pages that have any of those tags"; _
L0012 a special tag "=" adds all the tags that are present in the current page ||
L0013 
L0014 ++ Item format
L0015 
L0016 You can define what NextPage / PreviousPage will display as it's result. You can use the format in the same way as in the [[[doc:listpages-module | ListPages module]]].
L0017 
L0018 ++ Examples
L0019 
L0020 +++ Example 1
L0021 
L0022 [[code]]
L0023 [[module NextPage by="title"]]
L0024 **Next documentation page:** %%linked_title%%
L0025 [[/module]]
L0026 
L0027 [[module PreviousPage by="title"]]
L0028 **Previous documentation page:** %%linked_title%%
L0029 [[/module]]
L0030 [[/code]]
L0031 
L0032 +++ Result 1
L0033 
L0034 [[module NextPage by="title"]]
L0035 **Next documentation page:** %%linked_title%%
L0036 [[/module]]
L0037 
L0038 [[module PreviousPage by="title"]]
L0039 **Previous documentation page:** %%linked_title%%
L0040 [[/module]]
L0041 
L0042 
L0043 or you can use this code to place the links on the left and right side of your page (blog-like):
L0044 
L0045 +++ Example 2
L0046 
L0047 [[code]]
L0048 [[div style="overflow: hidden"]]
L0049 
L0050 [[div style="overflow: hidden; float: left; clear: left"]]
L0051 [[module PreviousPage]]
L0052 Previous: %%linked_title%%
L0053 [[/module]]
L0054 [[/div]]
L0055 
L0056 [[div style="overflow: hidden; float: right"]]
L0057 [[module NextPage]]
L0058 Next: %%linked_title%%
L0059 [[/module]]
L0060 [[/div]]
L0061 
L0062 [[/div]]
L0063 [[/code]]
L0064 
L0065 +++ Result 2
L0066 
L0067 [[div style="overflow: hidden"]]
L0068 
L0069 [[div style="overflow: hidden; float: left; clear: left"]]
L0070 [[module PreviousPage by"title"]]
L0071 Previous: %%linked_title%%
L0072 [[/module]]
L0073 [[/div]]
L0074 
L0075 [[div style="overflow: hidden; float: right"]]
L0076 [[module NextPage by="title"]]
L0077 Next: %%linked_title%%
L0078 [[/module]]
L0079 [[/div]]
L0080 
L0081 [[/div]]
```
