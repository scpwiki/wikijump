# Wikidot API: pages.get_meta

- Feature ID: `api-pages-get-meta`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `pages.get_meta` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:124` through line 153 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:124` through line 153  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0124 ++ pages.get_meta
L0125 
L0126 Get a bunch of pages and returns (some of) their meta data
L0127 
L0128 * argument keys:
L0129  * **site**: site to list pages from, eg. "my-site"
L0130  * **pages**: list of page full names to list (maximum 10 pages)
L0131 * returns dictionary of pages. For each page there will be item in the dictionary with page name as key and dictionary of the following page properties as value:
L0132  * //fullname//
L0133  * //created_at//
L0134  * //created_by//
L0135  * //updated_at//
L0136  * //updated_by//
L0137  * //title//
L0138  * //parent_fullname//
L0139  * //tags// -- list of all tags (including those starting with underscore)
L0140  * //rating//
L0141  * //revisions//
L0142  * **[PLANNED]** //comments// -- number of comments
L0143  * **[PLANNED]** //files// -- number of files attached to the page
L0144  * **[PLANNED]** //children// -- number of children pages
L0145 
L0146 [[code type="Python"]]
L0147 >>> s.pages.meta({"site": "my-site", "pages": ["blog:last-post", "blog:second-post"]})
L0148 {
L0149  "blog:last-post": {"fullname": "blog:last-post", "created_at": "2010-08-04T23:20:50Z", "created_by": "Gabrys", "updated_at": "2010-08-04T23:23:31Z", "updated_by": "Gabrys", "title": "Last Post", "parent_fullname": None, "tags": ["blog", "last"], "rating": 8, "revisions": 2},
L0150  "blog:second-post": {"fullname": "blog:second-post", "created_at": "2010-08-03T22:52:10Z", "created_by": "Gabrys", "updated_at": "2010-08-03T22:52:10Z", "updated_by": "Gabrys", "title": "Second Post", "parent_fullname": None, "tags": ["blog", "second"], "rating": 1, "revisions": 1}
L0151 }
L0152 [[/code]]
L0153 
```
