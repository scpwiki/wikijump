# The 'pagepath' field type

- Feature ID: `data-forms-pagepath-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'pagepath' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:pagepath-field/source.wikidot.txt:1` through line 14 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:pagepath-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:pagepath-field/source.wikidot.txt:1` through line 14  
SHA-256 of complete source file: `c593f84de2fbc3e64107145e667c0ecb2dfa8b963ff3d3774db2faf433d9d27e`

```wikidot
L0001 Lets the user create and select from a page within a page tree; the 'path' is the list of all parents plus that page.  It is visualized as {{page / page / page / page}} with at each level, the option of viewing that page, changing the page, or adding a new child.  This does not affect the actual page parent, and a form can have many pagepath fields.  The pagepath field value is stored as a page full name.  Hidden pages are invisible to users when selecting and navigating the page tree.
L0002 
L0003 [[code]]
L0004  origin:
L0005    label: Origin
L0006    type: pagepath
L0007    category: band-origin
L0008 [[/code]]
L0009 
L0010 The specific properties you can use on a pagepath field:
L0011 
L0012 * **category**: specifies the category that holds the page tree.
L0013 * **default**: defines a default value for the field shown on new pages.
L0014 * **max-level**: sets the maximum number of levels that can be created in the pagepath tree.
```
