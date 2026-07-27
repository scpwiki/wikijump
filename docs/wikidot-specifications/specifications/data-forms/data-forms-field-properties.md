# Field Properties

- Feature ID: `data-forms-field-properties`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Field Properties”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:field-properties/source.wikidot.txt:1` through line 67 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:field-properties (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:field-properties/source.wikidot.txt:1` through line 67  
SHA-256 of complete source file: `75252a4bfd48db780f701ae8c8407929c748cf96983893b3cb116e6e764d90c3`

```wikidot
L0001 +++ Properties that apply to all field types
L0002 
L0003 ++++ [[# label]] The 'label' property
L0004 
L0005 If you specify a 'label' property then the field gets that text in the left column, or before the field for joined fields.  If you do not specify a label then the field has an empty space in the left column, or is squashed up after the previous field, for joined fields.  For example:
L0006 
L0007 [[code]]
L0008 [[form]]
L0009 fields:
L0010   address-line-1:
L0011     label: Address
L0012     width: 30
L0013   address-line-2:
L0014     width: 30
L0015   address-line-3:
L0016     width: 30
L0017 [[/form]]
L0018 [[/code]]
L0019 
L0020 ++++ [[# join]] The 'join' property
L0021 
L0022 If you specify 'join: true' then the field is placed after the previous field, if any.  This property has no effect if the field is the first in the form.  For example:
L0023 
L0024 [[code]]
L0025 [[form]]
L0026 fields:
L0027   city:
L0028     label: City
L0029     width: 20
L0030   postcode:
L0031     label: Postcode
L0032     width: 8
L0033     join: true
L0034 [[/form]]
L0035 [[/code]]
L0036 
L0037 ++++ [[# before]]The 'before' property
L0038 
L0039 Provides a string of plain text that displays before the field value
L0040 
L0041 [[code]]
L0042 [[form]]
L0043 fields:
L0044   phone:
L0045     label: Phone
L0046     width: 10
L0047     before: +(1)
L0048 [[/form]]
L0049 [[/code]]
L0050 
L0051 ++++ [[# after]] The 'after' property
L0052 
L0053 Provides a string of plain text that displays after the field value
L0054 
L0055 [[code]]
L0056 [[form]]
L0057 fields:
L0058   speed:
L0059     label: Car speed
L0060     width: 4
L0061     after: kph
L0062 [[/form]]
L0063 [[/code]]
L0064 
L0065 +++ Properties for specific field types
L0066 
L0067 There are additional properties that only apply to specific field types. These are documented below with the field type(s) they apply to.
```
