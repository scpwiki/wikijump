# CSS Module

- Feature ID: `module-css`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `CSS` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:css-module/source.wikidot.txt:1` through line 36 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:css-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:css-module/source.wikidot.txt:1` through line 36  
SHA-256 of complete source file: `49d91080173204ee0dc2861f68975f4d9b61a1e35968bec86976634a3fb61b2d`

```wikidot
L0001 The CSS module lets you insert CSS code into a wiki page.  This is particularly useful for cross-site include (CSI) packages that need to use custom styling for their code.  When you use the CSS module in a CSI, that CSS code will be included in all pages that use the CSI.
L0002 
L0003 The syntax for the CSS module is:
L0004 
L0005 [[code]]
L0006 [[module CSS arguments...]]
L0007 CSS code
L0008 [[/module]]
L0009 [[/code]]
L0010 
L0011 ++ Example
L0012 
L0013 This example hides the side-bar menu on a single page (on themes with an elastic main-content):
L0014 
L0015 [[code]]
L0016 [[module CSS]]
L0017 #side-bar { 
L0018     visibility: hidden; 
L0019     width: 0
L0020 }
L0021 [[/module]]
L0022 [[/code]]
L0023 
L0024 ++ Arguments
L0025 
L0026 You can render the module's CSS code on the page in a {{@@[[code type="css"]]@@}} block by adding:
L0027 
L0028 * **show="true"**
L0029 
L0030 You can disable the module's CSS code so that it doesn't affect the theme by adding:
L0031 
L0032 * **disable="true"**
L0033 
L0034 ++ Multiple CSS modules
L0035 
L0036 A single page can contain any number of CSS modules.  Their code will be output, in order, in the page's HTML header.
```
