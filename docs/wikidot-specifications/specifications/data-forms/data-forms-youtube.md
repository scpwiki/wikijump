# YouTube and other external content

- Feature ID: `data-forms-youtube`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “YouTube and other external content”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:youtube/source.wikidot.txt:1` through line 32 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:youtube (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:youtube/source.wikidot.txt:1` through line 32  
SHA-256 of complete source file: `6679a47e05bc4842285ac2582bf85d420451574889bc9ed8e654e713c6815a6f`

```wikidot
L0001 +++ Data form field
L0002 To upload a YouTube video to your data form you need to use a **wiki** field. The user pastes the html embed code into the field on the dat aform.
L0003 
L0004 +++ Layout
L0005 To display it, above the @@====@@  separator use @@[[html]]@@ tags and form_raw as follows:
L0006 
L0007 [[code]]
L0008 [[html]]
L0009 %%form_raw{field}%%
L0010 [[/html]]
L0011 [[/code]]
L0012 
L0013 __Example__
L0014 
L0015 * add a wiki field to the data form:
L0016 [[code]]
L0017   bandvideo:
L0018     label: Video
L0019     type: wiki
L0020 [[/code]]
L0021 * above the separator you add an @@[[html]]@@ block and @@%%form_raw{bandvideo}%%@@ to display the video:
L0022 [[code]]
L0023 [[html]]
L0024 %%form_raw{bandvideo}%%
L0025 [[/html]]
L0026 [[/code]]
L0027 
L0028 * the user pastes the YouTube embed code into the field:
L0029 [[image df_video.jpg]]
L0030 
L0031 * the result is
L0032 [[image df_video2.jpg]]
```
