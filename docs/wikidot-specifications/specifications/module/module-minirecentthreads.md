# MiniRecentThreads Module

- Feature ID: `module-minirecentthreads`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `MiniRecentThreads` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:minirecentthreads-module/source.wikidot.txt:1` through line 18 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:minirecentthreads-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:minirecentthreads-module/source.wikidot.txt:1` through line 18  
SHA-256 of complete source file: `7915a0a578e57cdfa96482f7a9f7805fa573f22e212f43c18187767d40b63b2f`

```wikidot
L0001 ++ Description
L0002 
L0003 Displays most recent forum threads in a forum suitable to be included e.g. within the welcome page. The list items contain thread title, date started and number of posts.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || {{limit}} || no || positive integer || 5 || how many items to display? ||
L0009 
L0010 ++ Examples
L0011 
L0012 [[code]]
L0013 ++ Most recent forum threads
L0014 
L0015 [[module MiniRecentThreads limit="3"]]
L0016 [[/code]]
L0017 
L0018 simply displays 3 most recent forum threads.
```
