# Members Module

- Feature ID: `module-members`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Members` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:members-module/source.wikidot.txt:1` through line 31 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:members-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:members-module/source.wikidot.txt:1` through line 31  
SHA-256 of complete source file: `2bc653e3565a950d4685fb5810707412d6113c8045f071f70a2874cffcaa0973`

```wikidot
L0001 ++ Description
L0002 
L0003 This module is used to list members of the site.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || group || no || "members" _
L0009 "admins" _
L0010 "moderators" || "members" || limits the list to the specified group ||
L0011 || showSince || no || "no" or "false" || "yes" for group="members" || does not show the date joined; valid only for group="members" ||
L0012 || order || no || "userId", "userIdDesc", "joined", "joinedDesc", "name", "nameDesc" || "joined" || sort Members by name (alphabetically), by user ID or date of joining ||
L0013 
L0014 ++ Examples
L0015 
L0016 List all members of the site:
L0017 
L0018 [[code]]
L0019 [[module Members]]
L0020 [[/code]]
L0021 
L0022 List only site administrators:
L0023 
L0024 [[code]]
L0025 [[module Members group="admins"]]
L0026 [[/code]]
L0027 
L0028 List only moderators:
L0029 [[code]]
L0030 [[module Members group="moderators"]]
L0031 [[/code]]
```
