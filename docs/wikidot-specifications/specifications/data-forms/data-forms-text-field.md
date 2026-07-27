# The 'text' field type

- Feature ID: `data-forms-text-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'text' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:text-field/source.wikidot.txt:1` through line 33 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:text-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:text-field/source.wikidot.txt:1` through line 33  
SHA-256 of complete source file: `dea1716a94d6095efb82253efe22ea28169fb3c18d664765dd82e1baf2a630ac`

```wikidot
L0001 Defines a text or text box field.  Allows 'width' and 'height' as properties.  If you don't specify a height you get a normal 1-line text field.  If you do specify it, you get a text box.  For example:
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   name:
L0007     label: Your name
L0008     type: text
L0009     width: 30
L0010   comment:
L0011     label: Your comment
L0012     type: text
L0013     width: 50
L0014     height: 3
L0015   email:
L0016     label: email address
L0017     match: /^[_a-zA-Z0-9\-\+]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/
L0018 [[/form]]
L0019 [[/code]]
L0020 
L0021 The specific properties you can use on a text field:
L0022 
L0023 * **width**: specifies the visible field width in columns (fixed spaced characters, more or less).
L0024 * **height**: specifies the field height in rows, 1 is normal text field, 2 or more is a text box.
L0025 * **match**: specifies a regular expression (regex) that the field value must match.
L0026 * **match-error**: specifies a custom error message.
L0027 * **hint**: provides a string of text that is displayed in the field when empty.
L0028 * **default**: defines a default value for the field shown on new pages.
L0029 
L0030 In the hint, if you want to use special characters like a # then you need to escape the character using \. For example, **hint: enter a colorname like white or a hex value like \#468259**
L0031 
L0032 
L0033 Wiki syntax does not work in a text field.
```
