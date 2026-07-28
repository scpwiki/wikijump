# The 'hidden' field type

- Feature ID: `data-forms-hidden-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'hidden' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:hidden-field/source.wikidot.txt:1` through line 14 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:hidden-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:hidden-field/source.wikidot.txt:1` through line 14  
SHA-256 of complete source file: `d1c39d49764e2a4f8909e5cdb09f237157938b23393b6ac79cc9d8250c01fa13`

```wikidot
L0001 Adds data to the form that the user cannot see or edit. It takes no space visually.  This is for putting data into the page so that data can be used later.  The value of the field is defined by the 'value' property.
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   version:
L0007     type: hidden
L0008     value: 1.0
L0009 [[/form]]
L0010 [[/code]]
L0011 
L0012 The specific properties you can use on a hidden field:
L0013 
L0014 * **value**: sets the value of the field
```
