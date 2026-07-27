# Styling the output of a field

- Feature ID: `data-forms-output-style`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Styling the output of a field”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:output-style/source.wikidot.txt:1` through line 31 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:output-style (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:output-style/source.wikidot.txt:1` through line 31  
SHA-256 of complete source file: `fe5136aee4a388dbe6e4e3884c1d2fbb0c0d8f6a6c2a6d6430bbdeed4a697d44`

```wikidot
L0001 You can set the color and other styles of a field on the form after it is saved. Create the field in your data form in the normal way as follows:
L0002 
L0003 [[code]]
L0004 [[form]
L0005 fields
L0006 ...
L0007 ...
L0008   priority:
L0009     label: Priority
L0010     type: select
L0011     values:
L0012       normal: Normal
L0013       urgent: Urgent
L0014       critical: Critical
L0015 ....
L0016 [[/form]]
L0017 [[/code]]
L0018 
L0019 Above the @@====@@ separator add a CSS module:
L0020 
L0021 [[code]]
L0022 [[module css]]
L0023 .normal { color: green; }
L0024 .urgent { color: red; }
L0025 .critical { color: red; font-weight: bold;}
L0026 [[/module]]
L0027 [[/code]]
L0028 
L0029 Then use a css span class and a combination of form_raw and form_data to display the field in the relevant color:
L0030 
L0031 @@[[span class="%%form_raw{priority}%%"]]%%form_data{priority}%%[[/span]]@@
```
