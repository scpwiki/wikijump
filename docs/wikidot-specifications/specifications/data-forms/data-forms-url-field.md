# The 'url' field type

- Feature ID: `data-forms-url-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'url' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:url-field/source.wikidot.txt:1` through line 21 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:url-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:url-field/source.wikidot.txt:1` through line 21  
SHA-256 of complete source file: `9cf37f0da2d28aabe879e5e619907ebcc8ebaa7f0837ac3049089c7ad3379653`

```wikidot
L0001 This lets the user enter URLs. This is displayed as a link.
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   info_link:
L0007     type: url
L0008     default: ftp://example.com/files/
L0009     match-error: Custom error msg.
L0010     required: true
L0011     default-schema: ftp://
L0012 [[/form]]
L0013 [[/code]]
L0014 
L0015 The specific properties you can use on a url field:
L0016 
L0017 * **width**: specifies the visible field width in columns (fixed spaced characters, more or less).
L0018 * **default**: defines a default value for the field shown on new pages.
L0019 * **default-schema**: define a default schema for URL ('http://' if not specified).
L0020 * **match-error**: specifies a custom error message.
L0021 * **required**: specifies if the field is mandatory [true/false] ('false' if not specified).
```
