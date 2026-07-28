# FlickrGallery Module

- Feature ID: `module-flickrgallery`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `FlickrGallery` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:flickrgallery-module/source.wikidot.txt:1` through line 43 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:flickrgallery-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:flickrgallery-module/source.wikidot.txt:1` through line 43  
SHA-256 of complete source file: `2b55a822ff4cdccd2cbf647e60164c81dfa9077e2dd8e67b867124ea5e896f0f`

```wikidot
L0001 ++ Description
L0002 
L0003 Pulls images from [http://www.flickr.com Flickr] - online photo management.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || userName || no || any valid __flickr__ //user name// || none || limits results to a single user ||
L0009 || tags || no || tags || none || comma-delimited list of //tags// ||
L0010 || tagMode || no || {{"any"}}, {{"all"}} || {{"any"}} || applies OR, AND for tag sellection ||
L0011 || sort || no || {{"date-posted-asc"}} _
L0012 {{"date-posted-desc"}} _
L0013 {{"date-taken-asc"}} _
L0014 {{"date-taken-desc"}} _
L0015 {{"interestingness-desc"}} _
L0016 {{"interestingness-asc"}} _
L0017 {{"relevance"}} || {{"date-posted-desc"}} || sets the sort order ||
L0018 ||||||||||~ alternative attributes (do not play with these above) ||
L0019 || photosetId || no || any valid //photoset id// || none || gets images from a photoset ||
L0020 || groupId || no || any valid //group name// || none || gets images from a specified group ||
L0021 || groupUrl || no || URL address of the group main page || none || gets images from a specified group ||
L0022 ||||||||||~ display options ||
L0023 || perPage || no || any number between 1 and 100 || 30 || how many photos per page ||
L0024 || limitPages || no || any positive number || none || limits number of pages to navigate; also useful if you do not want to navigate pages at all ({{limitPages="1"}})||
L0025 || size || no || {{"square"}} - 75x75 pixels _
L0026 {{"thumbnail"}} -  100 on longest side _
L0027 {{"small"}} - 240 on longest side _
L0028 {{"medium"}}, 500 on longest side || {{"thumbnail"}} || size of the images to display ||
L0029 ||||||||||~ other options ||
L0030 || disableBrowsing || no || "yes"/"true" || none || disables displaying images in overlay windows when clicked||
L0031 || contentType || no || photos, screenshots, other, photos-screenshots, screenshots-other, photos-other, all || {{"all"}} || sets the type of images retrieved from Flickr ||
L0032 
L0033 ++ Examples
L0034 
L0035 Get pictures that have both "linux" and "sun" tags:
L0036 
L0037 [[code]]
L0038 [[module FlickrGallery tags="linux,sun" tagMode="all"]]
L0039 [[/code]]
L0040 
L0041 How it works:
L0042 
L0043 [[module FlickrGallery tags="linux,sun" tagMode="all"]]
```
