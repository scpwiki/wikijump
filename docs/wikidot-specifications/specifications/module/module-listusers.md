# ListUsers Module

- Feature ID: `module-listusers`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `ListUsers` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listusers-module/source.wikidot.txt:1` through line 35 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:listusers-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listusers-module/source.wikidot.txt:1` through line 35  
SHA-256 of complete source file: `ee11b0e4eece4cedde096bbd6f573bf2589e28feb73544e5a63688943b86f9b0`

```wikidot
L0001 The ListUsers module produces formatted output that lets you report on a set of users working with a site.
L0002 
L0003 The current implementation of the module outputs a block of text for the currently logged user only.
L0004 
L0005 [[code]]
L0006 [[module ListUsers users="."]]
L0007 module body
L0008 [[/module]]
L0009 [[/code]]
L0010 
L0011 [[include :www:doc-include:note-template-in-modules]]
L0012 
L0013 ||~ Variable ||~ Description ||
L0014 || number || The current user's ID number ||
L0015 || title || The current user's title or name ||
L0016 || name || The current user's name in unix format (lowercase, no spaces) ||
L0017 
L0018 + Example:
L0019 
L0020 
L0021 [[code]]
L0022 [[module ListUsers users="."]]
L0023 **You are user number %%number%%, %%title%% (%%name%%)!**
L0024 [[/module]]
L0025 [[/code]]
L0026 
L0027 + In action:
L0028 
L0029 [[module ListUsers users="."]]
L0030 **You are user number %%number%%, %%title%% (%%name%%)!**
L0031 [[/module]]
L0032 
L0033 This code prints nothing if user is anonymous.
L0034 
L0035 To comment or discuss on the planned design for this module [http://projects.wikidot.com/thread:129 please visit the projects forum].
```
