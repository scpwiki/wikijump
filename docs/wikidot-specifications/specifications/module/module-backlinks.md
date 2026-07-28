# Backlinks Module

- Feature ID: `module-backlinks`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Backlinks` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### Backlinks, Categories, and PageTree render live navigation-list DOM and legacy argument quirks

- Observation ID: `navigation-list-modules-live-dom-and-argument-quirks`
- Classification: `documentation-correction`
- Observed at: `2026-07-28`
- Analysis: The Backlinks documentation correctly says the module has no attributes; live Wikidot ignores both an undocumented page argument and unknown arguments and always lists pages linking to the containing page. The Categories documentation says includeHidden accepts true, but live Wikidot enables hidden categories for any non-empty double-quoted exact-case includeHidden value with no whitespace around '='; false, yes, TRUE, and True all enable it, while empty, bare, single-quoted, spaced, or uppercase attribute forms do not. Categories output uses Wikidot's category toggler DOM and writes a literal 1 inside the category-pages-<id>-options div while includeHidden is active. PageTree live output confirms root, showRoot, and depth semantics, case-sensitive attribute names, explicit root subtree selection, and plain ul/li/a tree markup.

Normative behavior:

- Backlinks recognizes [[module Backlinks]] and has no live-supported attributes.
- Backlinks ignores page and unknown arguments; every invocation targets the containing page.
- Backlinks lists anonymous-viewable pages in the current site that link to the containing page, sorted by live Wikidot's title order in observed fixtures.
- Backlinks renders div.backlinks-module-box containing a ul of li anchors; an empty result renders an empty backlinks-module-box.
- Categories recognizes module-name casing variants such as [[module categories ...]] and [[module CATEGORIES ...]].
- Categories hides underscore-prefixed categories by default, while _default remains visible.
- Categories enables hidden categories only when the source contains exact-case includeHidden with no whitespace around '=' and a non-empty double-quoted value. The value is not parsed as a boolean; includeHidden="false" and includeHidden="yes" enable hidden categories, while includeHidden="", includeHidden=true, INCLUDEHIDDEN="true", includeHidden = "false", and includeHidden='false' do not.
- Categories emits one div per selected category with h3 category text, an a#category-pages-toggler-<id> href='javascript:;' calling WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, <id>), and hidden category-pages plus category-pages-options divs.
- When includeHidden is active, live Wikidot writes the text 1 inside every category-pages-<id>-options div.
- PageTree uses the containing page as the default root, excludes the root unless showRoot="true", and limits traversal depth when depth is a positive integer.
- PageTree attribute names are case-sensitive; Showroot and Depth are ignored.
- PageTree root="<page>" selects that page as the subtree root, and showRoot="true" displays that explicit root as the first li.
- PageTree renders plain nested ul/li/a markup without wrapper classes or data attributes in observed live output.

Evidence:

- `install/local/wikidot-verification/artifacts/navigation-list-modules-live.json` (SHA-256 `07228f5618e06dba6cf90779bcd0bd2bcd834c1bc4a3a141f4528135fe4da5b1`), cases: `backlinks-default-current-page`, `backlinks-page-argument-ignored`, `backlinks-unknown-argument-ignored`, `pagetree-depth-and-showroot`, `pagetree-case-sensitive-arguments-and-explicit-root`, `categories-default-and-includehidden`, `categories-includehidden-argument-edge-cases`



## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:backlinks-module/source.wikidot.txt:1` through line 44 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:backlinks-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:backlinks-module/source.wikidot.txt:1` through line 44  
SHA-256 of complete source file: `f83ea064c1e512cf54858ef684ba4f882817c3279a4d5a7fda4f0648bbb02f3b`

```wikidot
L0001 ++ Description
L0002 
L0003 This module simply lists all the pages from the given wiki that contain links to the current page. 
L0004 
L0005 ++ Attributes
L0006 
L0007 No attributes.
L0008 
L0009 ++ Appearance 
L0010 
L0011 If you want to change the appearance of the list you should define (within your custom theme) the following class definition:
L0012 
L0013 [[code type="css]]
L0014 div.backlinks-module-box{
L0015     [your definition]
L0016 }
L0017 [[/code]]
L0018 
L0019 ++ Examples
L0020 
L0021 [[code]]
L0022 [[module Backlinks]]
L0023 [[/code]]
L0024 
L0025 and this produces backlinks for this page:
L0026 [[module Backlinks]]
L0027 
L0028 **Tip:** You can use the Backlinks module to make //soft categories// -- simply create pages in the namespace {{category:}}, e.g.
L0029 
L0030 * {{category:cars}}
L0031 * {{category:bikes}}
L0032 * etc...
L0033 
L0034 Each of these pages could have a description of the category and the {{@@[[module Backlinks]]@@}} that would list the pages...
L0035 
L0036 And within the pages you want to add to specific categories you would put links to these categories, e.g. at the bottom:
L0037 
L0038 [[code]]
L0039 ++ Categories:
L0040 
L0041 [[[category:cars]]], [[[category:bikes]]]
L0042 [[/code]]
L0043 
L0044 Moreover if all your category pages, i.e. {{category:cars}}, {{category:bikes}} etc. include a link to a page called {{category:all}} -- it is a quick way to put the module in the {{category:all}} too and have a list of all categories.
```
