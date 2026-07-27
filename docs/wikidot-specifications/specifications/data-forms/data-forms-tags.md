# Tags

- Feature ID: `data-forms-tags`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Tags”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

## Implementation contract

- Category templates MUST recognize the documented field and layout syntax.
- Create and edit flows MUST validate, normalize, store, and redisplay field values as documented.
- Page rendering, template variables, CSS hooks, ListPages selection, and ordering MUST expose stored values as documented.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Data-form template parsing and saved page rendering
- Public create/edit/view flow and ListPages query behavior where documented

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:tags/source.wikidot.txt:1` through line 3 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:tags (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:tags/source.wikidot.txt:1` through line 3  
SHA-256 of complete source file: `ebe65510abc2c4d451674ea28d2f398d4916bd169e1067197fafbf8f9c4e7b46`

```wikidot
L0001 It is not currently possible to set tags when saving the data form based on the values in the data form.
L0002 
L0003 However a workaround is possible until a tag field is implemented. This workaround is described at *http://community.wikidot.com/forum/t-402555/automatically-setting-tags-for-a-page-based-on-form-input
```
