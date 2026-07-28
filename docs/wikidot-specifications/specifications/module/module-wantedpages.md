# WantedPages Module

- Feature ID: `module-wantedpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `WantedPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### OrphanedPages and WantedPages render site-level link graph reports with live Wikidot table and pager markup

- Observation ID: `link-modules-live-orphaned-and-wanted-pages`
- Classification: `documentation-clarification`
- Observed at: `2026-07-28`
- Analysis: The OrphanedPages and WantedPages documentation names the selected link-graph concepts but does not define output DOM, ordering, visibility filtering, grouping, or WantedPages pagination. Controlled run-owned sandbox pages show that OrphanedPages lists viewable existing pages with no incoming internal page-link connections from other pages, renders an h1 heading followed by anchor rows with gray slug spans and br separators, and excludes an existing page that has an incoming internal link. WantedPages groups viewable source pages by missing target slug, excludes links whose target page exists, renders div.wanted-pages-module containing a table.form.grid with source-page anchors in the first cell and a class="newpage" missing-target anchor in the second cell, sorts the first page by wanted target slug and source title, renders 50 wanted targets per page, and emits top and bottom javascript: pager controls when more than one page exists.

Normative behavior:

- OrphanedPages recognizes the bare [[module OrphanedPages]] invocation and accepts no documented required attributes.
- OrphanedPages selects existing pages in the current site that are viewable by the anonymous viewer and have no incoming internal link connection from another page.
- OrphanedPages excludes an existing page once another page links to it with Wikidot's internal page-link syntax.
- OrphanedPages output begins with h1 text 'List of orphaned pages' and then emits one row per selected page as an anchor to /<page-slug>, a gray span containing the slug in parentheses, and a br separator.
- OrphanedPages rows are ordered by live Wikidot's case-insensitive title order with the slug as a deterministic tie-breaker in observed fixtures.
- WantedPages recognizes the bare [[module WantedPages]] invocation and accepts no documented required attributes.
- WantedPages selects internal page-link targets in the current site that do not currently exist, grouped by missing target slug.
- WantedPages excludes links to existing targets from the wanted-target column.
- WantedPages filters source pages through anonymous page-view permissions before listing them.
- WantedPages output emits div.wanted-pages-module containing a table.form.grid with headers 'Linked from' and 'Linked to (wanted page name)'.
- Each WantedPages row lists one or more source page anchors followed by br elements in the first cell and a class='newpage' anchor to the missing target slug in the second cell.
- WantedPages orders the first page by missing target slug; source pages inside a target row are ordered by case-insensitive source title in observed fixtures.
- WantedPages renders 50 wanted targets per page. When more than one page exists, live Wikidot emits matching top and bottom div.pager controls whose links call WIKIDOT.modules.WantedPagesModule.updateList(page, this).

Evidence:

- `install/local/wikidot-verification/artifacts/orphanedpages-module-live.json` (SHA-256 `e2d2da50f7f76fa4661322db8c9467d198e4e65b4b998d3ca13ea75c7e793f92`), cases: `orphanedpages-default`
- `install/local/wikidot-verification/artifacts/wantedpages-module-live.json` (SHA-256 `0d191ee1083e1322329ebcbaa4cae866b309cc683b32b44e13bd86bfb2263abf`), cases: `wantedpages-grouped-first-page`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:wantedpages-module/source.wikidot.txt:1` through line 8 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:wantedpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:wantedpages-module/source.wikidot.txt:1` through line 8  
SHA-256 of complete source file: `506e564b16e0b9f124099a4a032819614dd64d8de5c720dbd06849faed25870c`

```wikidot
L0001 This module lists all pages that do not exist but there are links that point to them.
L0002 
L0003 The same module can be found in **site tools** at the bottom of every page.
L0004 
L0005 +++ Usage
L0006 [[code]]
L0007 [[module WantedPages]]
L0008 [[/code]]
```
