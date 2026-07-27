# WhoInvited Module

- Feature ID: `module-whoinvited`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `WhoInvited` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:whoinvited-module/source.wikidot.txt:1` through line 22 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:whoinvited-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:whoinvited-module/source.wikidot.txt:1` through line 22  
SHA-256 of complete source file: `b3ee37c7cbb80be5a0218398b7aa848a39375645e975cbb384d8c5d40d1589da`

```wikidot
L0001 ++ Description
L0002 
L0003  This module will allow your users to look up how particular Members joined this Wiki. In particular it will display a chain of invitations. 
L0004 
L0005 
L0006 ++ Attributes
L0007 
L0008 No attributes required.
L0009 
L0010 ++ Examples
L0011 
L0012 [[code]]
L0013 Check how particular Members joined this Wiki.
L0014 
L0015 [[module WhoInvited]]	
L0016 [[/code]]
L0017 
L0018 And looks like this (this will not work very well within this Wiki because there are not many members here):
L0019 
L0020 Check how particular Members joined this Wiki.
L0021 
L0022 [[module WhoInvited]]
```
