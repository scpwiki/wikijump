# The 'select' field type

- Feature ID: `data-forms-select-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'select' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:select-field/source.wikidot.txt:1` through line 54 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:select-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:select-field/source.wikidot.txt:1` through line 54  
SHA-256 of complete source file: `d6afefccc3fb8c76e975406910395b46d07138bc6dde085383560b06a240c6f9`

```wikidot
L0001 Defines a multi-value selection field.  Requires a set of values.  If you specify two to four values, you get a horizontal radio field.  If you specify five or more values, you get a drop-down select field.  For example:
L0002 
L0003 [[code]]
L0004 [[form]]
L0005   type:
L0006     label: Music type
L0007     type: select
L0008     values:
L0009       0: Classical
L0010       1: Country
L0011       2: Folk
L0012       3: Indie
L0013       4: Jazz
L0014       5: Pop
L0015       6: Rock
L0016     default: 6
L0017 [[/form]]
L0018 [[/code]]
L0019 
L0020 
L0021 In the above example the properties of the select field were 0 to 6 with the default property of 6 which set the value to Rock. However, you can use words as properties, for example:
L0022 
L0023 [[code]]
L0024   type:
L0025     label: Music type
L0026     type: select
L0027     values:
L0028       cl: Classical
L0029       co: Country
L0030       fk: Folk
L0031       in: Indie
L0032       jz: Jazz
L0033       po: Pop
L0034       ro: Rock
L0035     default: ro
L0036 [[/code]]
L0037 
L0038 The specific properties you can use on a select field:
L0039 
L0040 * **default**: defines a default value for the field shown on new pages. For example **default:1**
L0041 
L0042 **Reserved values in a select field:**
L0043 
L0044 The values of **Yes**, **No**, **True** and **False** are reserved values that have a special meaning in the YAML code that powers data forms. To use them in your data form you need to place them inside quotemarks as follows otherwise they will not work:
L0045 
L0046 [[code]]
L0047   done:
L0048     label: Done?
L0049     type: select
L0050     values:
L0051       not: "No"
L0052       done: "Yes"
L0053     default: not
L0054 [[/code]]
```
