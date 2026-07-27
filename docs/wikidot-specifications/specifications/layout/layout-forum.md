# Forum layout structure

- Feature ID: `layout-forum`
- Category: `layout`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Render forum layout structure with the documented placeholders, conditional sections, element order, identifiers, and nesting.

## Implementation contract

- The Wikidot layout renderer MUST emit the documented regions, identifiers, order, and nesting.
- Conditional regions and placeholders MUST use the documented context and visibility rules.
- Browser tests MUST verify final DOM and any user-visible intermediate state.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms
- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:117` through line 145 (canonical)

## Documentation-derived behavioral evidence

### doc:layout-reference (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:layout-reference/source.wikidot.txt:117` through line 145  
SHA-256 of complete source file: `bdb2ffc85a5b5e200b2df4a63c32fe5a86a2699a5c8ce58678103af949ab93ba`

```wikidot
L0117 + Forum structure
L0118 
L0119 Forum elements are embedded within the {{div#page-content}} element. Only elements below this one are listed.
L0120 
L0121 ++ Forum start view (list of groups and categories)
L0122 
L0123 * {{div.forum-start-box}}
L0124  * {{div.forum-group}} - for each of the groups (top-level forum structures)
L0125   * {{div.head}}
L0126    * {{div.title}}
L0127    * {{div.description}}
L0128   * {{div}}
L0129    * {{table}}
L0130     * {{tr.head}} - description of fields
L0131      * {{td}}
L0132      * {{td}}
L0133      * {{td}}
L0134      * {{td}}
L0135     * {{tr}} - for each of categories in the group
L0136      * {{td.name}}
L0137       * {{div.title}}
L0138       * {{div.description}}
L0139      * {{td.threads}}
L0140      * {{td.posts}}
L0141      * {{td.last}}
L0142 
L0143 ++ Category view (list of threads)
L0144 
L0145 ++ Thread view
```
