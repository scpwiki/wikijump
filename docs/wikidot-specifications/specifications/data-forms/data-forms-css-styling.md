# CSS Styling

- Feature ID: `data-forms-css-styling`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “CSS Styling”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:css-styling/source.wikidot.txt:1` through line 28 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:css-styling (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:css-styling/source.wikidot.txt:1` through line 28  
SHA-256 of complete source file: `6d5bbab7ed8e48bb11d571db3adc725ebd6da5986d6ca174ad240c72d0d69deb`

```wikidot
L0001 You can modify the look and feel of your data forms using CSS (either per-site, or per page using the [http://www.wikidot.com/doc:css-module CSS module].  This is the CSS model for data forms:
L0002 
L0003 * **table** _
L0004   //class//: form-table
L0005  * **tr** _
L0006      //class//: form-row  row-{row number}
L0007  * **td** _
L0008      //class//: form-labels
L0009  * **span** _
L0010       //class//: form-label
L0011  * **td** _
L0012      //class//: form-values
L0013  * **span/div** (div for wiki and static) _
L0014       //class//: form-value field-{name} _
L0015       //class//': form-error (added to field while save when there is matching error)
L0016   * **{field}** _
L0017        //class//: form-{type}
L0018   * **span** _
L0019        //class//: form-message
L0020 
L0021 +++ Styling the hint text
L0022 If you have a long hint text you might find that it is longer than the text box. This is because by default the text box is a partcular width. In this case you can either set the width of that particular field to be wider or you can use CSS to set the same width for all text input boxes and ensure the hint fits inside it by using:
L0023 
L0024 [[code type="css"]]
L0025 input[type="text"], textarea {
L0026     width:100%;
L0027 }
L0028 [[/code]]
```
