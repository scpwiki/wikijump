# RatedPages Module

- Feature ID: `module-ratedpages`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `RatedPages` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### RatedPages renders live top-rated DOM and accepts rating/order filter forms

- Observation ID: `ratedpages-basic-live-order-filter-and-dom`
- Classification: `documentation-clarification`
- Observed at: `2026-07-29`
- Analysis: The RatedPages documentation lists ordering, rating filters, limit, comments, and examples, but omits the generated DOM and contains an example order spelling, rate-asc, that is not present in the allowed-values table. Controlled run-owned sandbox pages in a plus/minus rating category show that live Wikidot renders a top-rated pages box with list-item rows, displays page titles linked by full slug, prints an inline gray rating label, defaults to rating-desc, accepts rating-asc and the undocumented example spelling rate-asc, and applies minRating and maxRating inclusively.

Normative behavior:

- RatedPages expands as a self-contained [[module RatedPages ...]] marker; the rendered page must not leak the raw module syntax.
- The outer output is div.top-rated-pages-box containing div.top-rated-pages-list and one div.list-item per row.
- Each row links to /<page-fullname>, displays the page title as the anchor text, and appends span style="color: #777" containing (Rating: N).
- Omitting order behaves as order="rating-desc".
- order="rating-asc" sorts lower ratings before higher ratings.
- order="rate-asc" is accepted as a compatibility spelling for rating ascending, despite not appearing in the allowed-values table.
- order="date-created-desc" sorts newer pages before older pages.
- minRating is inclusive and limits rows to pages whose rating is equal to or above the supplied integer.
- maxRating is inclusive and limits rows to pages whose rating is equal to or below the supplied integer.
- limit bounds the number of rendered rows.

Evidence:

- `install/local/wikidot-verification/artifacts/ratedpages-live-basic.json` (SHA-256 `f332646cec4e0ba108f1a8fe961e594adf92fb1808a5875db70d2712a2a31602`), cases: `default`, `rating-asc`, `rate-asc`, `date-created-desc`, `min-rating`, `max-rating`

### RatedPages comments attribute uses final exact non-empty value

- Observation ID: `ratedpages-comments-live-truthiness-and-output`
- Classification: `documentation-correction`
- Observed at: `2026-07-29`
- Analysis: The RatedPages documentation says comments accepts true/yes, but controlled live pages show a broader legacy rule. Live Wikidot ignores an omitted comments attribute, comments="", a bare comments token, and an uppercase Comments key. For exact lowercase comments with a quoted value, the final duplicate value controls display, and any non-empty value enables the comments label, including false, no, and uppercase TRUE.

Normative behavior:

- Omitting comments renders only the rating label.
- The exact lowercase comments attribute appends , Comments: N inside the same gray rating span when its final quoted value is non-empty.
- comments="true", comments="yes", comments="false", comments="no", and comments="TRUE" all enable the comments label.
- comments="" disables the comments label.
- A bare comments token does not enable the comments label.
- An uppercase Comments key is ignored.
- When comments appears more than once with the exact lowercase spelling, the final exact value wins; a final empty value disables the label and a final non-empty value enables it.

Evidence:

- `install/local/wikidot-verification/artifacts/ratedpages-live-comments.json` (SHA-256 `770c4f5300f7d8a9277bff3fe3d0c989d96b37aab293cee7dc540bd038c12a17`), cases: `omitted`, `comments-true`, `comments-yes`, `comments-false`, `comments-no`, `comments-empty`, `comments-uppercase-value`, `comments-uppercase-key`, `comments-bare`, `comments-true-then-no`, `comments-no-then-true`, `comments-true-then-empty`, `comments-empty-then-true`, `comments-false-then-empty`, `comments-empty-then-false`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:ratedpages-module/source.wikidot.txt:1` through line 38 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:ratedpages-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:ratedpages-module/source.wikidot.txt:1` through line 38  
SHA-256 of complete source file: `5a8fdb32861838d117ee6226b305cc86166ecea30312e79cc8d980693b847159`

```wikidot
L0001 ++ Description
L0002 
L0003 Displays top-rated pages. Also a category can be specified to limit the results.
L0004 
L0005 [[note]]
L0006 This module is under development and not yet complete. 
L0007 But should work as described below.
L0008 [[/note]]
L0009 
L0010 ++ Attributes
L0011 
L0012 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0013 || {{category}} || no || valid category name || none || limits reported pages to a single category ||
L0014 || {{order}} || no || "date-created-asc", "date-created-desc", "rating-asc", "rating-desc" || "rating-desc" || ||
L0015 || {{minRating}} || no || integer || none || limits the results to the pages having rating equal or above this limit ||
L0016 || {{maxRating}} || no || integer || none || limits the results to the pages having rating equal or below this limit ||
L0017 || {{limit}} || no || positive integer || 10 || limits number of displayed pages ||
L0018 || {{comments}} || no || "true"/"yes" || none || display number of comments too ||
L0019 
L0020 ++ Examples
L0021 
L0022 Display top-rated pages from the category {{rateit}}
L0023 
L0024 [[code]]
L0025 [[module RatedPages category="rateit" limit="20" comments="true" minRating="0"]]
L0026 [[/code]]
L0027 
L0028 Display "most hated" pages:
L0029 
L0030 [[code]]
L0031 [[module RatedPages category="rateit" order="rate-asc" limit="20" comments="true" maxRating="-1"]]
L0032 [[/code]]
L0033 
L0034 Display new submissions:
L0035 
L0036 [[code]]
L0037 [[module RatedPages category="rateit" order="date-created-desc" limit="20" comments="true"]]
L0038 [[/code]]
```
