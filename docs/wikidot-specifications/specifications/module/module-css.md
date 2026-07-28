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

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### CSS module show and disable arguments use exact legacy boolean syntax

- Observation ID: `css-module-live-argument-and-output-semantics`
- Classification: `documentation-clarification`
- Observed at: `2026-07-28`
- Analysis: The CSS module documentation lists show="true" and disable="true" but does not define alternate true values, case sensitivity, duplicate arguments, or exact output placement. Controlled run-owned sandbox pages show that live Wikidot removes every CSS module from the page source stream, emits active CSS modules into the HTML head in source order, and renders a visible code block only for exact lowercase show="true" or show="yes". disable uses the same exact lowercase true/yes values and suppresses the head CSS while still allowing a visible show code block. Uppercase keys, uppercase values, single quotes, bare flags, and whitespace around '=' are ignored for these flags. Duplicate exact arguments use the last exact occurrence.

Normative behavior:

- CSS recognizes complete [[module CSS ...]]...[[/module]] blocks and removes the module delimiters and CSS body from the normal page-content stream.
- By default, a CSS module contributes its body to the page's compiled body styles / HTML head and renders no visible page-content code.
- Multiple active CSS modules contribute head styles in source order.
- show enables a visible CSS code block only when the final exact lowercase show argument has value "true" or "yes" with double quotes and no whitespace around '='.
- show values "false", "", uppercase "TRUE", a bare show token, a single-quoted value, whitespace around '=', or uppercase argument keys do not render a visible code block.
- disable suppresses the head CSS only when the final exact lowercase disable argument has value "true" or "yes" with double quotes and no whitespace around '='.
- disable values "false", "", uppercase "TRUE", a bare disable token, a single-quoted value, whitespace around '=', or uppercase argument keys leave the CSS active.
- When both show and disable are active, the CSS body is rendered as a visible code block but is not contributed to the page's head styles.
- For repeated exact show or disable arguments, the last exact occurrence controls that flag.

Evidence:

- `install/local/wikidot-verification/artifacts/css-module-live.json` (SHA-256 `f3db293f0fb0912aa3c0c2f55fbbb4b8bd6e596f349569f257f62555e97a4438`), cases: `plain`, `show-true`, `show-yes`, `show-uppercase-value`, `show-false`, `show-empty`, `show-bare`, `disable-true`, `disable-yes`, `disable-uppercase-value`, `disable-false`, `disable-empty`, `disable-bare`, `show-disable`, `uppercase-show-key`, `uppercase-disable-key`, `show-true-then-false`, `show-false-then-true`, `disable-true-then-false`, `disable-false-then-true`, `show-spaced-equals`, `disable-spaced-equals`, `show-single-quoted`, `disable-single-quoted`



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
