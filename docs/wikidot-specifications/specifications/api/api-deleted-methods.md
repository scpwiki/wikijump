# Removed Wikidot API methods

- Feature ID: `api-deleted-methods`
- Category: `api`
- Documentation status: `documented-negative`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Reject or omit API methods that the documentation explicitly records as deleted.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:239` through line 248 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:239` through line 248  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0239 ++ Deleted methods
L0240 
L0241 The following methods are deleted and don't work anymore:
L0242 
L0243 * site.pages
L0244 * site.categories
L0245 * page.get
L0246 * page.files
L0247 * page.save
L0248 * user.sites
```
