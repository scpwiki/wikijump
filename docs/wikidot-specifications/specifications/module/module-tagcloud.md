# TagCloud Module

- Feature ID: `module-tagcloud`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `TagCloud` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### TagCloud renders live DOM anchors, URL arguments, style interpolation, error blocks, and legacy 3D SWF wrapper

- Observation ID: `tagcloud-live-dom-url-style-and-legacy-3d`
- Classification: `documentation-correction`
- Observed at: `2026-07-29`
- Analysis: The TagCloud documentation lists the public arguments but does not define the emitted DOM, malformed-value fallbacks, error text, style interpolation order, hidden-tag sort key, or 3D runtime markup, and its default color table labels are reversed compared with live Wikidot. Controlled run-owned sandbox pages show that live 2D TagCloud emits direct HTML anchors inside div.pages-tag-cloud-box, filters counts by category when supplied, sorts tags alphabetically after removing leading underscores from the sort key, applies limit to the sorted display slice before style interpolation, excludes underscore-prefixed tags unless showHidden has any non-empty value, and builds generated links from target plus tag/category path arguments. Live default 2D colors are light rgb(128,128,192) for the least-common displayed count and dark rgb(64,64,128) for the most-common displayed count. Single-sided font or color overrides are ignored, mismatched paired font units and invalid paired colors emit error-block divs, and mode="3d" emits the legacy SWFObject wrapper with WP-Cumulus tagcloud variables.

Normative behavior:

- TagCloud accepts documented arguments mode, maxFontSize, minFontSize, maxColor, minColor, limit, target, category, showHidden, urlAttrPrefix, skipCategoryFromUrl, width, and height.
- A 2D TagCloud render emits div.pages-tag-cloud-box containing one a.tag anchor per displayed tag; it does not emit Wikidot div/span source syntax.
- When category is supplied, tag counts are limited to pages in that category and generated links include /category/<category> unless skipCategoryFromUrl is enabled.
- When category is omitted, visible tags are counted site-wide and generated links omit category path arguments.
- Tags are displayed in alphabetical tag-name order after removing leading underscores from the sort key. Hidden tags keep their original underscore in rendered text, hrefs, and 3D encoded anchors. limit is applied to that sorted list before font and color interpolation.
- limit values that are zero, negative, or non-integer fall back to the default limit of 50. A positive limit displays at most that many tags.
- Tags beginning with underscore are hidden by default. Any non-empty showHidden value, including false, no, true, and yes, includes hidden tags; an empty showHidden value keeps hidden tags excluded.
- The target argument changes the generated link page. urlAttrPrefix prefixes generated tag and category argument names, producing segments such as /lp_tag/<tag>/lp_category/<category>.
- skipCategoryFromUrl values true and yes omit the generated category argument when category is supplied.
- Live default style endpoints are minFontSize 100%, maxFontSize 300%, least-common color rgb(128,128,192), and most-common color rgb(64,64,128). This reverses the color endpoint labels in the documentation table.
- Custom paired minFontSize and maxFontSize values are accepted only when both are present and use the same unit from px, pt, em, or %. A single-sided font-size override is ignored. Mismatched or invalid paired units render the error text 'Format for minFontSize and maxFontSize must be the same (px, em, pt or %).' inside div.error-block.
- Custom paired minColor and maxColor values are accepted only when both are present and both parse as three decimal 0-255 RGB channels. A single-sided color override is ignored. Invalid paired colors render the error text 'Unsupported color format. Use "RRR,GGG,BBB" for Red,Green,Blue each within 0-255 range.' inside div.error-block.
- mode="3d" emits div.pages-tag-cloud-box containing a script tag for the legacy CloudFront swfobject.js asset, a generated flashcontent div, and JavaScript constructing SWFObject('/common--javascript/tagcloud/tagcloud.swf', 'tagcloud', width, height, '7', '#FFFFFF').
- 3D TagCloud maps the most-common color to tcolor and hicolor, maps the least-common color to tcolor2, encodes tag anchors with location.protocol plus location.hostname, and interpolates WP-Cumulus style weights from 12 to 30 across the displayed count range.
- width and height apply to the 3D SWFObject dimensions; omitted values use the documented default of 300.

Evidence:

- `install/local/wikidot-verification/artifacts/tagcloud-module-live.json` (SHA-256 `83fdb631a88eab55d45dd385e49b52ccc339a72d7ceaa6e50d5c08c3a8ace5cf`), cases: `category-default`, `category-show-hidden-true`, `custom-target-prefix-skip-category`, `custom-style`, `limit-two`, `invalid-limit-zero-and-partial-style`
- `install/local/wikidot-verification/artifacts/tagcloud-module-edge-live.json` (SHA-256 `9c9fe558a91a51fd786c3b6fdb886e1d797bbec97fe61a4390f62d2dfe0f2f27`), cases: `omitted-category`, `show-true`, `show-yes`, `show-false`, `show-no`, `show-empty`, `limit-zero`, `limit-neg`, `limit-text`, `limit-one`, `target-with-category`, `prefix-with-category`, `skip-true-no-prefix`, `only-max-font`, `mismatch-font-units`, `only-min-color`, `bad-color`, `mode-3d`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:tagcloud-module/source.wikidot.txt:1` through line 42 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:tagcloud-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:tagcloud-module/source.wikidot.txt:1` through line 42  
SHA-256 of complete source file: `dc709f8de09fe3cf9ad75d975b0224b8c7ff186fa0e639a392fc8769773f9cee`

```wikidot
L0001 ++ Description
L0002 
L0003 This module creates a customizable //tag cloud// for tagged pages. Tag Cloud module can be used both for creating plain tag cloud or 3D tag cloud.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || {{mode}} || no || 3d || none || turns on 3D mode ||
L0009 || {{maxFontSize}} || no || any //length// value (in units of px, pt, em or %) || 300% || font size for the most common tags ||
L0010 || {{minFontSize}} || no || any //length// value (in units of px, pt, em or %) || 100% || font size for the least common tags ||
L0011 || {{maxColor}} || no || color definition {{RRR, GGG, BBB}} each in range 0-255 || {{128,128,192}} || color for the most common tags ||
L0012 || {{minColor}} || no || color definition {{RRR, GGG, BBB}} each in range 0-255 || {{64,64,128}} || color for the least common tags ||
L0013 || {{limit}} || no || any integer > 0 || 50 || how many tags should be displayed ||
L0014 || {{target}} || no || valid page name || {{system:page-tags}} || name of the target page where the links should lead to ||
L0015 || {{category}} || no || valid category name || none || limits tags (and displayed pages when the tag is clicked) to the specified page category) ||
L0016 || {{showHidden}} || no || true / yes || false || shows hidden tags, i.e. these starting with underscore (_) ||
L0017 || {{urlAttrPrefix}} || no || any string || none || adds a given string as a prefix to parameters in URLs generated by each tag; it makes the module more compatible with [[[doc:listpages-module | ListPages]]] and [[[doc:pagecalendar-module | PagesCalendar]]] modules ||
L0018 || {{skipCategoryFromUrl}} || no || true / yes || false || by default if you use {{category}} parameter, it is also added as a parameter in URLs generated by the tags; if this option is enabled, name of the category is skipped ||
L0019 || {{width}} || no ||  integer || 300 || sets width of the 3D Tag Cloud ||
L0020 || {{height}} || no ||  integer || 300 || sets height of the 3D Tag Cloud ||
L0021 
L0022 ++ Notes
L0023 
L0024 If you want to change font sizes, both {{maxFontSize}}  and {{minFontSize}} must be defined and use the same size units.
L0025 
L0026 If you want to change tag colors, both {{maxColor}}  and {{minColor}} must be defined.
L0027 
L0028 If you choose a custom target page make sure a module {{PagesByTag}} is there - otherwise there would be no point in linking the tags.
L0029 
L0030 By default tags starting with the underscore "_" are hidden and are not shown by default. Use {{showHidden="true"}} to display them.
L0031 
L0032 The 3D tag representation is based on the [http://wordpress.org/extend/plugins/wp-cumulus/ WP-Cumulus WordPress plugin].
L0033 
L0034 ++ Examples
L0035 
L0036 [[code]]
L0037 [[module TagCloud minFontSize="1em" maxFontSize="2em" maxColor="8,8,8" minColor="200,200,228" category="news"]]
L0038 [[/code]]
L0039 
L0040 A working example of a 3D tag cloud can be found on our [http://snippets.wikidot.com/code:3d-tagcloud Snippet pages].
L0041 
L0042 You can also put your TagCloud in the side menu (often located in page {{nav:side}}. This is sometimes nice ;-)
```
