# PageCalendar Module

- Feature ID: `module-pagecalendar`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `PageCalendar` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### PageCalendar renders live calendar DOM, target/date URLs, category selection, @URL propagation, and tag-filter quirks

- Observation ID: `pagecalendar-live-dom-url-and-tag-filter-quirks`
- Classification: `documentation-correction`
- Observed at: `2026-07-29`
- Analysis: The PageCalendar documentation describes category and tag selectors but does not define emitted DOM, link ordering, selected-state classes, duplicate-argument precedence, empty or missing category errors, or the observed tag behavior. Controlled run-owned sandbox pages show that PageCalendar emits a div.page-calendar-box with nested ul/li anchors grouped by creation year and month, uses targetPage or startPage for generated date links, prefixes generated argument names with urlAttrPrefix, accepts comma- and whitespace-separated category lists, and treats duplicate category attributes with last-value precedence. An empty or nonexistent explicit category renders Wikidot's error-block text. Contrary to the documentation, the tags attribute does not filter calendar counts in the observed live cases; it is propagated into generated /tag/ or prefixed /PREFIX_tag/ path arguments with plus signs replaced by spaces while preserving other punctuation such as commas. A category="@URL|fallback" selector reads prefixed category URL path arguments and carries the resolved URL category forward in generated links, while the fallback category is not carried as a generated category segment. Live date URL navigation marks the selected year li for /date/YYYY and the selected month li for /date/YYYY.M.

Normative behavior:

- PageCalendar accepts documented arguments category, tags, startPage, targetPage, and urlAttrPrefix.
- A PageCalendar render emits div.page-calendar-box containing a nested ul tree. Top-level li entries represent years and nested li entries represent months.
- Years and months are ordered descending by creation date. Anchor labels are '<year> (<count>)' and '<English month name> (<count>)'. Month date path values use an unpadded numeric month such as 2026.7.
- Omitted category defaults to the current page category. The current page itself is counted when it belongs to that category and is viewable.
- category accepts comma- and whitespace-separated category names. '*' selects the whole site. An empty or nonexistent explicit category renders div.error-block with text 'The requested categories do not (yet) exist.'.
- targetPage and startPage select the page used as the generated-link target. targetPage='' falls back to the current page. startPage is a compatibility alias for targetPage.
- urlAttrPrefix prefixes generated argument names, producing date segments such as /lp_date/2026 and /lp_date/2026.7.
- Duplicate category attributes use last-value precedence in the observed live fixture.
- In observed live behavior, tags does not filter calendar counts despite the documentation saying it is a selection criterion.
- When tags is present, PageCalendar carries the tag expression into generated links before date segments using the singular argument name tag, or PREFIX_tag with urlAttrPrefix. Plus signs in the tag expression are replaced by spaces in the generated href, while other punctuation such as commas is preserved.
- category='@URL|fallback' and tags='@URL|fallback' read prefixed URL path arguments when urlAttrPrefix is present. A category value resolved from the URL is carried forward in generated links before the date segment; a fallback category is not carried forward.
- When the current URL supplies date YYYY, the matching year li has class='selected'. When the URL supplies date YYYY.M, the matching month li has class='selected' and the year li is not selected.

Evidence:

- `install/local/wikidot-verification/artifacts/pagecalendar-module-live.json` (SHA-256 `79f4ff732fb44d6a5cb73f4e51492e36a8830b2cfc3b554a58bb62c40233ba99`), cases: `explicit-category`, `target-prefix`, `startpage-alias`, `tags-space`, `tags-comma`, `multi-category-space`, `multi-category-comma`, `empty-category`, `duplicate-category`, `empty-target`, `url-default-no-args`, `url-category-path`, `url-tags-path`, `default-current-category`, `default-target-prefix`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagecalendar-module/source.wikidot.txt:1` through line 41 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:pagecalendar-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:pagecalendar-module/source.wikidot.txt:1` through line 41  
SHA-256 of complete source file: `7a10b6fc5c70acbc335e20f400e980affe8df73ea95ac668bc145af7d17356bd`

```wikidot
L0001 ++ Description
L0002 
L0003 The PageCalendar module creates a blogger-friendly calendar that displays the number of pages (articles) created per year and month. It also works great with the [[[doc:listpages-module |ListPages module]]] as shown below.
L0004 
L0005 [[note]]
L0006 We are still working on this module so the final syntax and specifications might change.
L0007 [[/note]]
L0008 
L0009 ++ Attributes
L0010 
L0011 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0012 || {{category}} || no || comma- or space-separated category names or "*" for whole wiki, or @URL || current category || sets the scope of processed pages ||
L0013 || {{tags}} || no || comma- or space-separated tag names with {{+}} and {{-}} modifiers, _
L0014 or {{@URL}} || none || lists tags that are used as a criteria to select pages, the "+" before the tag makes it required, "-" means "without a tag" and tags without modifiers translate to "pages that have any of those tags"; _
L0015 "@URL" takes the tags from the URL ||
L0016 || {{startPage}} or {{targetPage}} || no || any valid wiki page || current page || sets the page that will be displayed when any of the dates is clicked ||
L0017 || {{urlAttrPrefix}} || no || any alphanumeric || none || prefix for the parameters passed via the URL e.g. to the [[[doc:listpages-module |ListPages module]]] ||
L0018 
L0019 If you are using PageCalendar with ListPages, make sure that the {{urlAttrPrefix}} has the same value in both modules.
L0020 
L0021 Parameters that accept the {{@URL}} value, i.e. allow for passing the value in the URL, also allow for default values similar to the [[[doc:listpages-module#toc8 | ListPages module]]].
L0022 
L0023 ++ Examples
L0024 
L0025 List pages in the documentation section grouped by year and month, linked with the ListPages module:
L0026 
L0027 [[code]]
L0028 [[module PageCalendar category="doc"]]
L0029 [[module ListPages category="doc" perPage="7" date="@URL" separate="false" prependLine="||~ Page||~ Date created||~ Created by ||"]]
L0030 || %%linked_title%% || %%date%% || %%author%% ||
L0031 [[/module]]
L0032 [[/code]]
L0033 
L0034 [[div style="overflow: hidden; width: 55em;"]]
L0035 [[div style="float: left; margin: 0 7em 0 0"]]
L0036 [[module PageCalendar category="doc"]]
L0037 [[/div]]
L0038 [[module ListPages category="doc" perPage="7" date="@URL" separate="false" prependLine="||~ Page||~ Date created||~ Created by ||"]]
L0039 || %%linked_title%% || %%date%% || %%author%% ||
L0040 [[/module]]
L0041 [[/div]]
```
