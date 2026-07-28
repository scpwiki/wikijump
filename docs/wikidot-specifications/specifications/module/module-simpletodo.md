# SimpleToDo Module

- Feature ID: `module-simpletodo`
- Category: `module`
- Documentation status: `documented-deprecated`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement Wikidot's deprecated SimpleToDo list module, including task mutation, attributes, permissions, and rendered controls.

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

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:simpletodo-module/source.wikidot.txt:1` through line 40 (canonical)

## Documentation-derived behavioral evidence

### doc:simpletodo-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:simpletodo-module/source.wikidot.txt:1` through line 40  
SHA-256 of complete source file: `05b6d46e5cbd05f748f32ec9426628f33ea1d8fd3bdc21459b3b92f7833994e3`

```wikidot
L0001 ++ Description
L0002 
L0003 This module lets you create simple "todo" lists with items to check/uncheck/reorder. Installation is as easy as placing a one-line code in a page, i.e.
L0004 
L0005 [[code]]
L0006 [[module SimpleToDo id="mylist1"]]
L0007 [[/code]]
L0008 
L0009 ++ Attributes
L0010 
L0011 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0012 || {{id}} || yes || any string || none || identifies the list within a given site ||
L0013 
L0014 The list is identified by its {{id}} attribute. You can have several lists in one page with different ids, and have the same list on different pages if you use the same id.
L0015 
L0016 There is no "save" button in the list, it is saved in real-time as you change it.
L0017 
L0018 ++ Quickstart
L0019 
L0020 * to add an item, click "+ add item"
L0021 * to edit an item, click on it, to save hit enter or click outside the input box
L0022 * to reorder items, simply drag&drop
L0023 * to edit list's title, click on it
L0024 * to remove item, click on the red cross
L0025 * to add a link to an item, click the "pen&link" icon; destination can be an url (e.g. http://slashdot.org) or unix-name of a page within a wiki (e.g. {{start}}).
L0026 * to remove a link, edit it and remove the text in the input box
L0027 * 
L0028 ++ Permissions
L0029 
L0030 The list can be altered by anyone who has __edit__ permissions on the page the list is embedded in.
L0031 
L0032 ++ Examples
L0033 
L0034 By simply copying the example code from above a list can be created:
L0035 
L0036 [[module SimpleToDo id="mylist1"]]
L0037 
L0038 (most likely you are not allowed to edit this list)
L0039 
L0040 The SimpleToDo module has been developed by [[user e1n]].
```
