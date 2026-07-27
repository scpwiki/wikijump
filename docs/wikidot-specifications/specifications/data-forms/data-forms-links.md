# Links

- Feature ID: `data-forms-links`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Links”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:links/source.wikidot.txt:1` through line 19 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:links (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:links/source.wikidot.txt:1` through line 19  
SHA-256 of complete source file: `3c27819475424ca44a7e16ad1462c18eafdce45430ff17fbff3f39e2d17588b7`

```wikidot
L0001 ++ [[# external]] External Links
L0002 
L0003 +++ Data form field
L0004 To upload a url to your data form you need to use a **url** field. It defaults to http:// format so the user just needs to enter the url in the format //www.wikidot.com//
L0005 
L0006 +++ Layout
L0007 To display the link, above the @@====@@  separator use @@%%form_data{field}%%@@.
L0008 
L0009 You can have the link open in a new window by adding a * as follows: @@*%%form_data{file}%%@@
L0010 
L0011 ------
L0012 
L0013 ++ [[# internal]] Internal Links
L0014 
L0015 +++ Data form field
L0016 To include an internal link in to your data form you use a **text** field. The user just enters the name of the page in the box on the form..
L0017 
L0018 +++ Layout
L0019 To display it, above the @@====@@  separator use normal internal link syntax and form_data:  @@[[[%%form_data{field}%%]]]@@
```
