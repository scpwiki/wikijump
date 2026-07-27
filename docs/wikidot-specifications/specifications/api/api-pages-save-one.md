# Wikidot API: pages.save_one

- Feature ID: `api-pages-save-one`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `pages.save_one` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

## Implementation contract

- The public API MUST accept the documented method name and parameter forms.
- Authentication, authorization, limits, filtering, ordering, return shapes, and errors MUST match the documented contract.
- Deleted methods MUST remain unavailable unless live compatibility evidence proves a later replacement.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Published Wikidot API method boundary
- Public persistence/query behavior reached through that method

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:176` through line 203 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:176` through line 203  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0176 ++ pages.save_one
L0177 
L0178 Save page. Site and page keys of argument array are required. Set specific keys to update the properties, omit to keep current values.
L0179 
L0180 * argument keys:
L0181  * **site**: site to save page to
L0182  * **page**: page full name to save
L0183  * **title** (optional): title to set
L0184  * **content** (optional): page content -- wiki source
L0185  * **tags** (optional): array of tags to set
L0186  * **parent_fullname** (optional): parent page full name, "-" to reset
L0187  * **save_mode** (optional): allowed mode of operation
L0188   * create -- only allow creating new objects (exception thrown if object with this name already exists)
L0189   * update -- only allow updating objects (exception thrown if no object with this name exists)
L0190   * create_or_update (default) -- allow both creating and updating object
L0191  * **rename_as** (optional): rename the page (in addition to possible other changes in source etc)
L0192  * **revision_comment** (optional): revision comment (displayed in history)
L0193  * **notify_watchers** (optional):
L0194   * true: notify watchers about the edit (as if it was done with the web interface)
L0195   * false (default): don't notify watchers
L0196 * returns: saved page (as in pages.get_one)
L0197 
L0198 [[note]]
L0199 We only want to make API for comments currently, but it will be expanded to both comments and forum. That's why the namespace is {{posts}} and not {{comments}}.
L0200 
L0201 **Note the API for {{posts}} namespace is not yet stable. This means it may change in the future without notice and break compatibility.**
L0202 [[/note]]
L0203 
```
