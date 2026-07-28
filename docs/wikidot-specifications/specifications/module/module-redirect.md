# Redirect Module

- Feature ID: `module-redirect`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Redirect` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:redirect-module/source.wikidot.txt:1` through line 34 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:redirect-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:redirect-module/source.wikidot.txt:1` through line 34  
SHA-256 of complete source file: `dd5226c1195c1cc557d0e48071950d1e7808fb28181d93219a1e480fba3c9773`

```wikidot
L0001 ++ Description
L0002 
L0003 The Redirect module performs a "301 Permanently Moved" redirection, i.e. it tells a web browser to request another web page.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || destination || yes || page-name or  URL || none || where to redirect?||
L0009 
L0010 When the {{destination}} attribute is just an alphanumeric string, e.g. "start", the page which contains the Redirect module will automatically forward the browser to the wiki page called "start". If destination is the whole URL address (e.g. "http://slashdot.org"), the browser will be redirected to this external address.
L0011 
L0012 ++ Mapping
L0013 
L0014 If the {{destination}} attribute ends with a slash, e.g. destination="start**/**" or destination="@@http://www.example.com@@**/**", the current URL will be mapped to the destination in the following way. The code for the module would be:
L0015 
L0016 [[code]]
L0017 [[module Redirect destination="http://www.example.com/base/"]]
L0018 [[/code]]
L0019 
L0020 Now if the Redirect module is placed on page @@http://your-wiki.wikidot.com/redir@@ the following mapping will be performed:
L0021 ||~ from ||~ to ||
L0022 || @@http://your-wiki.wikidot.com/redir@@ || @@http://www.example.com/base/@@ || 
L0023 || @@http://your-wiki.wikidot.com/redir/@@**mapped-path** || @@http://www.example.com/base/@@**mapped-path** ||
L0024 || @@http://your-wiki.wikidot.com/redir/@@**mapped-path/file1.html** || @@http://www.example.com/base/@@**mapped-path/file1.html** ||
L0025 
L0026 ++ Preventing the redirect
L0027 
L0028 If the Redirect module redirected the browser always there would be no way to edit the actual page. The solution is to pass an extra parameter to the module in the URL as follows:
L0029 
L0030 @@http://your-wiki.wikidot.com/page-with-redirect/@@**noredirect/true**
L0031 
L0032 There should be an information box where the module is placed.
L0033 
L0034 Working with the Redirect module might not be very convenient but even of you have to do this you will not configure it every day ;-)
```
