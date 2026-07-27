# Wikidot API: pages.get_one

- Feature ID: `api-pages-get-one`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `pages.get_one` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:154` through line 175 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:154` through line 175  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0154 ++ pages.get_one
L0155 
L0156 Gets one page and returns all its properties
L0157 
L0158 * argument keys:
L0159  * **site**: site to get a page from, e.g. "my-site"
L0160  * **page**: page full name to get, e.g. "start" or "blog:first-post"
L0161 * returns: page properties as dictionary (consult documentation of [[[doc:ListPages module]]])
L0162  * all those listed in {{pages.get_meta}} plus:
L0163  * //parent_title//
L0164  * //children//
L0165  * //content// -- page content, if page is assigned a form it's in YAML format
L0166  * //html// -- generated HTML of the page (as seen from browser excluding navigational bars etc)
L0167  * //comments// -- number of comments
L0168  * //commented_at//
L0169  * //commented_by//
L0170 
L0171 [[code type="Python"]]
L0172 >>> s.pages.get_one({"site": "my-site", "page": "blog:last-post"})
L0173 {"created_at": "2010-08-04T23:20:50Z", "created_by": "Gabrys", "updated_at": "2010-08-04T23:23:31Z", "updated_by": "Gabrys", "title": "Last Post", "parent_fullname": None, "tags": ["blog", "last"], "rating": 8, "revisions": 2, "parent_title": None, "children": 0, "content": "Test blog post", "html": "<p>Test blog post</p>", "comments": 0, "commented_at": None, "commented_by": None}
L0174 [[/code]]
L0175 
```
