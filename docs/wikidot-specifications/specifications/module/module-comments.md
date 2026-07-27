# Comments Module

- Feature ID: `module-comments`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `Comments` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:comments-module/source.wikidot.txt:1` through line 33 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:comments-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:comments-module/source.wikidot.txt:1` through line 33  
SHA-256 of complete source file: `3da747507b8de1d6fd17df24bd6429e506cdc632bf26b96d2ecfb74d2e4a1db2`

```wikidot
L0001 ++ Description
L0002 
L0003 Inserts page discussion below page contents. A very useful module if you want to comment contents of the page.
L0004 
L0005 By default, if the visitor has enough permissions, the form for comments is already open. This can be changed by setting the {{hideForm="true"}} attribute.
L0006 
L0007 ++ Attributes
L0008 
L0009 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0010 || title || no || text string  || "" || shows alternate heading for the comments block ||
L0011 || hide || no || {{"true"}} || {{"false"}} || hides the discussion and requires user click to show it ||
L0012 || hideForm || no || {{"true"}}, {{"yes"}} || {{"false"}} || does not display the open input form by default, just a link to add a comment ||
L0013 || order || no || {{"reverse"}}, {{"forwards"}} || forwards || If set to {{"reverse"}}, this shows comments in reverse order, newest above oldest ||
L0014 
L0015 ++ Examples
L0016 
L0017 Initially hidden discussion.
L0018 [[code]]
L0019 [[module Comments hide="true"]]
L0020 [[/code]]
L0021 
L0022 Full discussion within a page.
L0023 [[code]]
L0024 [[module Comments]]
L0025 [[/code]]
L0026 
L0027 Make the comments block be listed in [[toc]] (by disabling the default heading and insert a heading manually):
L0028 
L0029 [[code]]
L0030 + Comments
L0031 
L0032 [[module Comments]]
L0033 [[/code]]
```
