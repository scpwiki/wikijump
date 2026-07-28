# Wikidot API: pages.select

- Feature ID: `api-pages-select`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `pages.select` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:102` through line 123 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:102` through line 123  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0102 ++ pages.select
L0103 
L0104 Select pages that match given criteria
L0105 
L0106 * argument keys (if not stated otherwise, possible values documented in [[[doc:ListPages module]]]):
L0107  * **site**: site to get pages from, e.g. "my-site"
L0108  * **pagetype** (optional): default "*"
L0109  * **categories** (optional): list of category names to pull pages from, default: all categories
L0110  * **tags_any** (optional): list of tags, page must have at least one of them
L0111  * **tags_all** (optional): list of tags, page must have all of them
L0112  * **tags_none** (optional): list of tags, page must have none of them
L0113  * **parent** (optional): single page name or "-" for pages with no parent
L0114  * **created_by** (optional): single user name
L0115  * **rating** (optional)
L0116  * **order** (optional)
L0117 * returns: list of page full names
L0118 
L0119 [[code type="Python"]]
L0120 >>> s.pages.select({"site": "my-site", "categories": ["blog", "news"], "tags_none": ["_draft"], "order": "created_at desc"})
L0121 ["blog:last-post", "blog:second-post", "blog:first-post", "blog:_template"]
L0122 [[/code]]
L0123 
```
