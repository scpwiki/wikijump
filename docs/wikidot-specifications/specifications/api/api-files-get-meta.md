# Wikidot API: files.get_meta

- Feature ID: `api-files-get-meta`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `files.get_meta` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:33` through line 49 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:33` through line 49  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0033 ++ files.get_meta
L0034 
L0035 Get meta data of given files
L0036 
L0037 * argument keys:
L0038  * **site**: what site we ask about
L0039  * **page**: what page (full name) we ask about
L0040  * **files**: name of files to get meta data of -- max 10 of them
L0041 * returns: dictionary of files. File name is key of each item in it. The value is a dictionary of:
L0042  * //size// -- size in bytes
L0043  * //comment//
L0044  * //mime_type//
L0045  * //mime_description//
L0046  * //uploaded_by//
L0047  * //uploaded_at//
L0048  * //download_url// -- URL to download the file from. For private sites, the URL contains authorization token {{?ukey=@@..@@.}} valid for about 5 minutes.
L0049 
```
