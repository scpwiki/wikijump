# Wikidot API: posts.select

- Feature ID: `api-posts-select`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `posts.select` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:204` through line 215 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:204` through line 215  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0204 ++ posts.select
L0205 
L0206 Select post/comments on given site, page, thread and/or in reply to other comment.
L0207 
L0208 * argument keys:
L0209  * **site**: site to get pages to get comments from, e.g. "my-site"
L0210  * **page** (optional): page to get comments from
L0211  * **thread** (optional): thread to get posts from -- not yet implemented
L0212  * **reply_to** (optional): only select comments/posts that are direct replies to this one ("-" means not replies to other posts/comments)
L0213  * **created_by** (optional): select posts by this user
L0214 * returns: list of post/comments IDs sorted by date posted
L0215 
```
