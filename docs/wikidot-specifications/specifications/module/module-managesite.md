# ManageSite Module

- Feature ID: `module-managesite`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ManageSite` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:managesite-module/source.wikidot.txt:1` through line 17 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:managesite-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:managesite-module/source.wikidot.txt:1` through line 17  
SHA-256 of complete source file: `dcc3bcbe1ad729b3bdbc87a2d79f28e81b424f9cbd5f28d2ab4aaffdad29c3f7`

```wikidot
L0001 **This module is deprecated.**
L0002 
L0003 ++ Description
L0004 
L0005 The ManageSite module provides you with a panel to manage your site, users, and so on.  On new sites, the admin:manage page contains this module.  You can put this module on any page but only logged-in site administrators are allowed to use the panel.  Any other users will get a permission error.
L0006 
L0007 ++ Attributes
L0008 
L0009 This module does not take any attributes.
L0010 
L0011 ++ Examples
L0012 
L0013 Please this code on a page by itself:
L0014 
L0015 [[code]]
L0016 [[module ManageSite]]
L0017 [[/code]]
```
