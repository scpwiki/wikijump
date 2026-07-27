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
