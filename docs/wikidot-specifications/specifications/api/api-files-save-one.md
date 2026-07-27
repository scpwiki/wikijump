# Wikidot API: files.save_one

- Feature ID: `api-files-save-one`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `files.save_one` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:65` through line 91 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:65` through line 91  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0065 ++ files.save_one
L0066 
L0067 [[note]]
L0068 With this method you can attach files not bigger than 50MB. Other file size limits also apply:
L0069 
L0070 * site storage -- can't upload file bigger than current unused file storage for site
L0071 * maximum file size depending on free/Pro Wikidot plan
L0072 [[/note]]
L0073 
L0074 Attaches file to page
L0075 
L0076 * argument keys:
L0077  * **site**: site of page to attach file to
L0078  * **page**: page to attach file to
L0079  * **file**: name of file to attach
L0080  * **comment** (optional): file description
L0081  * **save_mode** (optional): allowed mode of operation
L0082   * create -- only allow creating new objects (exception thrown if object with this name already exists)
L0083   * update -- only allow updating objects (exception thrown if no object with this name exists)
L0084   * create_or_update (default) -- allow both creating and updating object
L0085  * **content**: base64-encoded file content
L0086  * **notify_watchers** (optional):
L0087   * true: notify watchers about the edit (as if it was done with the web interface)
L0088   * false (default): don't notify watchers
L0089  * **revision_comment** (optional): revision comment (displayed in history)
L0090 * returns: the newly uploaded file information as dictionary the same to what files.get_meta return for each file
L0091 
```
