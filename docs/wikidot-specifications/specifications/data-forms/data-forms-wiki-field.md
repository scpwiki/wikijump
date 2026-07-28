# The 'wiki' field type

- Feature ID: `data-forms-wiki-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'wiki' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:wiki-field/source.wikidot.txt:1` through line 15 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:wiki-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:wiki-field/source.wikidot.txt:1` through line 15  
SHA-256 of complete source file: `0f799742cbdae02eac942b09544ff311b4ef770f077c62ff85f23459fbe18ab8`

```wikidot
L0001 Works like text but lets the user enter wiki syntax. 
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   version:
L0007     label: Fancy text field
L0008     type: wiki
L0009 [[/form]]
L0010 [[/code]]
L0011 
L0012 The specific properties you can use on a wiki field:
L0013 
L0014 * **width**: specifies the width of the field in the dataform.
L0015 * **height**: specifies the height of the field in the dataform.
```
