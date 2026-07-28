# Using the data in ListPages modules

- Feature ID: `data-forms-dataforms-and-listpages`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Using the data in ListPages modules”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:dataforms-and-listpages/source.wikidot.txt:1` through line 9 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:dataforms-and-listpages (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:dataforms-and-listpages/source.wikidot.txt:1` through line 9  
SHA-256 of complete source file: `2b6da73c430f0723d90bd259c8ac295a26ece7c7f016f8d4e4f865cd0553f3de`

```wikidot
L0001 The data that is produced by data forms can be used in the ListPages module (*http://www.wikidot.com/doc:listpages-module). With the band example, a ListPages module could look like this:
L0002 
L0003 [[code]]
L0004 [[module ListPages category="band" order="name"  separate="false" prependLine="||~ Band||~ Type ||~ Current ||" appendLine="||||||||~ ||"]]
L0005 || %%title_linked%% || %%form_data{type}%% || %%form_data{current}%% ||
L0006 [[/module]]
L0007 [[/code]]
L0008 
L0009 [[image df_bandlist.jpg]]
```
