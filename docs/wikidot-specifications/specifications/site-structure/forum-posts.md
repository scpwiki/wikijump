# Forum posts and post layout

- Feature ID: `forum-posts`
- Category: `site-structure`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot site-structure capability “Forum posts and post layout”, including its identity, relationships, routes, and rendering implications.

## Implementation contract

- The persistence model MUST represent the documented entity and relationships.
- Public links, routes, selection behavior, permissions, and rendered structure MUST preserve those relationships.
- Imported Wikidot identifiers and URLs MUST remain compatibility-stable.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:83` through line 118 (canonical)

## Documentation-derived behavioral evidence

### doc:site-structure (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:site-structure/source.wikidot.txt:83` through line 118  
SHA-256 of complete source file: `20e91b5e74e135e07d4559a7057d2a43ce36b0e3db98fd3c8b20c10a5468b33f`

```wikidot
L0083 ++ Posts and posts layout
L0084 
L0085 Posts are smallest units here. They are just what people say. Posts can be edited after they are posted.
L0086 
L0087 Posts layout can be set using the [[[doc:ManageSite module]]] and can be:
L0088 
L0089 * flat/linear - posts appear one after another; it is not possible to reply to the post that is not the last post,
L0090 * nested - the tree-like structure, any post can be replied and new posts not necessarily appear at the end of the discussion but under the post that is being replied; //max nest level// determines number of possible levels and defaults to 2
L0091 
L0092 
L0093 [[div style="float:left; width: 43%; padding: 0 3%"]]
L0094 Example of flat structure:
L0095 * post 1
L0096 * post 2
L0097 * post 3
L0098 * post 4
L0099 * ...
L0100 [[/div]]
L0101 [[div style="float:left; width: 43%; padding: 0 3%"]]
L0102 Nested structure:
L0103 * post 1
L0104  * reply to post 1
L0105  * another reply to post 1
L0106 * post 2
L0107  * reply
L0108   * reply
L0109   * ...
L0110 [[/div]]
L0111 ~~~~~~~
L0112 
L0113 Flat/nested choice often determines the way people discuss. In the flat layout there is only one "path" - that is why this is called "linear". 
L0114 
L0115 Nested layout offers more freedom, more digressions and more paths in the discussion but it is more difficult to spot new posts (unless you use RSS feed or watch a thread).
L0116 
L0117 The default layout is: nested with max_nest_level = 2.
L0118 
```
