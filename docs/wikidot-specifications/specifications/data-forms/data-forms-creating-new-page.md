# Creating a new page

- Feature ID: `data-forms-creating-new-page`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Creating a new page”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:creating-new-page/source.wikidot.txt:1` through line 21 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:creating-new-page (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:creating-new-page/source.wikidot.txt:1` through line 21  
SHA-256 of complete source file: `1a5ae5d76bd97bf7148b42bc365c8f72be5f2f24e475a09e60f8a3896042da07`

```wikidot
L0001 You can create a new page in your data form category in three ways:
L0002 
L0003 1) in your browser address bar, enter the category and pagename after the sitename, for example @@http://yoursite.wikidot.com/@@**band:genesis**. Then press Enter.
L0004 
L0005 2) create a [*http://www.wikidot.com/doc:newpage-module NewPage module] button. This method allows you to set the category, parent page, any tags you want when the page is saved and the text of the button. for example:
L0006 
L0007 [[code]]
L0008 Enter the name of the band and press the button:
L0009 [[module NewPage size="30" category="band" parent="bands" tags="rock" button="Add a new rock band"]]
L0010 [[/code]]
L0011 
L0012 3) use the NewPage Button at *http://snippets.wikidot.com/code:newpage-button which is an excellent snippet created by [[*user james-kanjo]]. Using our band example, if you use this you will need to change the name of the band when you edit the form from //Band// to the actual name of the band.
L0013 
L0014 @@[[include :snippets:newpage-button@@
L0015 @@|size=30@@
L0016 @@|category=band@@
L0017 @@|name=band@@
L0018 @@|parent=bands@@
L0019 @@|tags=rock@@
L0020 @@|button=Add a new band@@
L0021 @@]]@@
```
