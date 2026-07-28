# MembershipByPassword Module

- Feature ID: `module-membershipbypassword`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `MembershipByPassword` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:membershipbypassword-module/source.wikidot.txt:1` through line 17 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:membershipbypassword-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:membershipbypassword-module/source.wikidot.txt:1` through line 17  
SHA-256 of complete source file: `45fd9123e359b8ca58fd4cc9054e949bcee8dc9ab7cd9f6a9e6c863ab9a86d23`

```wikidot
L0001 **This module is deprecated.  Use the [/doc:join-module Join module] instead.**
L0002 
L0003 ++ Description
L0004 
L0005 Allows registered users (these having a valid wikidot account) to automatically become members of the site provided they know the password. 
L0006 
L0007 The password itself must be configured using the [[[doc:managesite-module | ManageSite]]] module and the password-based membership sign-in must be enabled.
L0008 
L0009 ++ Attributes
L0010 
L0011 No attributes required.
L0012 
L0013 ++ Examples
L0014 
L0015 [[code]]
L0016 [[module MembershipByPassword]]
L0017 [[/code]]
```
