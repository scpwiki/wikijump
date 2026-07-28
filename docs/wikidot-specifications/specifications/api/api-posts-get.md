# Wikidot API: posts.get

- Feature ID: `api-posts-get`
- Category: `api`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `posts.get` API method with its documented arguments, authentication and permission requirements, limits, return values, and failure behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:216` through line 230 (canonical)

## Documentation-derived behavioral evidence

### doc:api (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:api/source.wikidot.txt:216` through line 230  
SHA-256 of complete source file: `3dcc770266cc7120e22a047a46208a9a718fac05ce17cac54d1b0127c50a17b8`

```wikidot
L0216 ++ posts.get
L0217 * argument keys:
L0218  * **site**: site to get comments from, e.g. "my-site"
L0219  * **posts**: list of IDs of posts/comments to get (max 10 of them)
L0220 * returns dictionary of posts/comments. For each post/comment there will be item in the dictionary with post/comment ID as key and and dictionary of the following post properties as value:
L0221  * //id// -- ID of post/comment
L0222  * //fullname// -- fullname of page to which comment belongs
L0223  * //reply_to// -- ID of comment which this post/comment replies to
L0224  * //title// -- title of the post/comment
L0225  * //content// -- post/comment body (wiki syntax)
L0226  * //html// -- post/comment body as HTML
L0227  * //created_by// -- user that posted post/comment
L0228  * //created_at// -- time post/comment was posted
L0229  * //replies// -- number of replies to given post/comment -- not yet implemented
L0230 
```
