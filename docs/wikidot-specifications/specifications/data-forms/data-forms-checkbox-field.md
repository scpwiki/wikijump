# The 'checkbox' field type

- Feature ID: `data-forms-checkbox-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'checkbox' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:checkbox-field/source.wikidot.txt:1` through line 18 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:checkbox-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:checkbox-field/source.wikidot.txt:1` through line 18  
SHA-256 of complete source file: `80246a710718a8a0c14a97dc164390e84fc7a6f58c251c18150b68afac5ed5be`

```wikidot
L0001 Defines a checkbox field, stored in the form data as 0 or 1.  For example:
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   onions:
L0007     label: Do you want onions?
L0008     type: checkbox
L0009   salami:
L0010     label: How about extra salami?
L0011     type: checkbox
L0012     default: 1
L0013 [[/form]]
L0014 [[/code]]
L0015 
L0016 The specific properties you can use on a checkbox field:
L0017 
L0018 * **default**: defines a default value for the field shown on new pages.
```
