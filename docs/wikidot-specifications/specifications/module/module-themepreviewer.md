# ThemePreviewer Module

- Feature ID: `module-themepreviewer`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ThemePreviewer` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:themepreviewer-module/source.wikidot.txt:1` through line 23 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:themepreviewer-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:themepreviewer-module/source.wikidot.txt:1` through line 23  
SHA-256 of complete source file: `7f652f40c0b29ecb19a7786747d11fa98dc5024fb3b6de53e97e7da29af14057`

```wikidot
L0001 ++ Description
L0002 
L0003 When inserted into the page source the module displays a list of available themes for a given site and allows previewing themes without entering site settings.
L0004 
L0005 ++ Attributes
L0006 
L0007 * noUi="true" - disable the user interface, and shows the theme specified on the URL.  Use this syntax to specify the theme to apply to the page:
L0008 
L0009 [[code]]
L0010 ?theme_url=http://example.com/style.css
L0011 [[/code]]
L0012 
L0013 ++ Examples
L0014 
L0015 Just do this:
L0016 
L0017 [[code]]
L0018 [[module ThemePreviewer]]
L0019 [[/code]]
L0020 
L0021 which results in (you can try this now):
L0022 
L0023 [[module ThemePreviewer]]
```
