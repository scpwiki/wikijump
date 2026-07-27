# The 'file' field type

- Feature ID: `data-forms-file-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'file' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:file-field/source.wikidot.txt:1` through line 21 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:file-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:file-field/source.wikidot.txt:1` through line 21  
SHA-256 of complete source file: `642006add71989a17c436937be1f28bfea0f75afa08b5fc777ab7446552d73bf`

```wikidot
L0001 This lets the user upload files directly from the data form. It is displayed as a link to the file.
L0002 
L0003 Files are not uploaded to the same page. Instead, a separate page is created for each file in a different category, 'file' by default, with the pagename being the name of the image.
L0004 
L0005 [[code]]
L0006 [[form]]
L0007 fields:
L0008   document:
L0009     type: file
L0010     label: Upload document
L0011     category: alternative-category
L0012 [[/form]]
L0013 [[/code]]
L0014 
L0015 The specific properties you can use on a file field:
L0016 
L0017 * **category**: specifies the category that the page will be created in ('file' category if not specified), and the uploaded file is attached to this page.
L0018 
L0019 [[note]]
L0020 Note that images won't be treated like they are when attaching an image to simple (i.e. non-data form enabled) page. This means they won't be displayed by the @@[[gallery]]@@ tag.
L0021 [[/note]]
```
