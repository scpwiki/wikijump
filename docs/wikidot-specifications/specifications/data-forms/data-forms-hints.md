# Hints & Tips

- Feature ID: `data-forms-hints`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Hints & Tips”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:hints/source.wikidot.txt:1` through line 5 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:hints (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:hints/source.wikidot.txt:1` through line 5  
SHA-256 of complete source file: `67382f94f3124d7095a5c54f079cf6596cfbd4ca5487f09e3f3f1ebcb15a169b`

```wikidot
L0001 * You can use dataform fields to set headings in your page.
L0002 * The wiki fieldtype is very powerful as it allows the user to use all the formatting and other wiki syntax. It is therefore more useful than the text fieldtype
L0003 * You can allow the user to set their own CSS for particular elements on the page by passing the contents of a dataform field into a css module.
L0004 
L0005 If you want to know how to achieve any of these or other dataform features on your site, please contact us on the community site at http://community.wikidot.com/forum:start
```
