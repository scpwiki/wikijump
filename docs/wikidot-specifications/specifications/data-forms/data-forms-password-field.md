# The 'password' field type

- Feature ID: `data-forms-password-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'password' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:password-field/source.wikidot.txt:1` through line 11 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:password-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:password-field/source.wikidot.txt:1` through line 11  
SHA-256 of complete source file: `a1f90005e77dbab4aa1c8d513348a8980980eb7fddaf61013feffb922c1ec7cf`

```wikidot
L0001 This lets the user enter masked text. To the user, each character they type is replaced by an asterisk ( * ).
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   pass:
L0007     type: password
L0008 [[/form]]
L0009 [[/code]]
L0010 
L0011 **Important:** Entered text is not encrypted, you can always read it in page source.
```
