# Search Module

- Feature ID: `module-search`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Search` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:search-module/source.wikidot.txt:1` through line 47 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:search-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:search-module/source.wikidot.txt:1` through line 47  
SHA-256 of complete source file: `1ddf6bb65b95619fb88db64e3547227bebd17d8871b48b434228e1f708b45831`

```wikidot
L0001 ++ Description
L0002 
L0003 The Search module lets your users search the current site.  You can place the Search module itself on any page, but your site //must// contain a page called "//search:site//" that (also) contains the Search module.
L0004 
L0005 ++ Attributes
L0006 
L0007 The Search module allows these attributes:
L0008 
L0009 * mini="true" - shows a simpler search box, hiding the radio button search options.
L0010 
L0011 ++ Example
L0012 
L0013 On your site's //start// page:
L0014 
L0015 [[code]]
L0016 ++ Search this site
L0017 [[module Search]]
L0018 = ([http://www.wikidot.com/doc:searching Search tips])
L0019 [[/code]]
L0020 
L0021 On your site's //search:site// page: 
L0022 
L0023 [[code]]
L0024 [[module Search]]
L0025 [[/code]]
L0026 
L0027 ++ Advanced Settings for search:site Page
L0028 This option only works on your site's //search:site// page in conjunction with the default search box (normally located near the top navigation bar).
L0029 The default source of this page is:
L0030 [[code]]
L0031 [[module Search]]
L0032 
L0033 [!-- please do not remove or change this page if you want to keep the search function working --] 
L0034 [[/code]]
L0035 You can edit the parameters of the Search module using the following options.
L0036 * a="p" sets the default search mode to pages only (default).
L0037 * a="f" sets the default search to forum only.
L0038 * a="pf" sets the default search to pages and forum.
L0039 Used in tandem with //mini="true"// you can hide the radio button options displaying a simpler interface and controlling the type of search done on your site.
L0040 
L0041 ++ Advanced Example
L0042 To limit searching to forums only and hide the search radio button options, edit your //search:site// page so it looks like this:
L0043 [[code]]
L0044 [[module Search a="f" mini="true"]]
L0045 
L0046 [!-- please do not remove or change this page if you want to keep the search function working --] 
L0047 [[/code]]
```
