# The Pagepath concept

- Feature ID: `data-forms-pagepath`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The Pagepath concept”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:pagepath/source.wikidot.txt:1` through line 55 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:pagepath (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:pagepath/source.wikidot.txt:1` through line 55  
SHA-256 of complete source file: `307d78e3a2f70a801d5b1a47ce95b3744e343e37fe5b2e467375fd63ba4f29c8`

```wikidot
L0001 Wikidot data forms have a unique concept, the Page Tree and pagepath, which is a way of organizing data.  It creates a page in a specific category for each pagepath entry you enter. Using our band example you could set the //origin// of the band.  Band origins form a tree:
L0002 
L0003 * _root
L0004  * USA
L0005   * Chicago
L0006   * Los Angeles 
L0007  * Australia
L0008   * Melbourne
L0009   * Sydney  
L0010  * Europe
L0011   * UK
L0012    * London
L0013      * North-London
L0014      * South-London
L0015       * Dulwich 
L0016      * East-London
L0017      * West-London
L0018    * Newcastle
L0019    * Glasgow 
L0020   * Germany
L0021   * Sweden
L0022 
L0023 Each part of the tree is a wiki page.  Imagine this tree is held in a category called **band-origin**.  Now we can use parent links to attach Dulwich to South-London to London to UK to Europe, and Chicago to USA etc.
L0024 
L0025 The Wikidot data form system makes it easy to navigate, and edit such a tree. You define a 'pagepath' field and tell it to use the **band-origin:** category, as shown in part of a dataform below:
L0026 
L0027 [[code]]
L0028  origin:
L0029    label: Origin
L0030    type: pagepath
L0031    category: band-origin
L0032 [[/code]]
L0033 
L0034 A page tree is always anchored to a page called _root that Wikidot creates automatically when you start using a page tree in forms.
L0035 
L0036 When you and your users are entering data into the dataform, for the pagepath field they will initially see a single dropdown box. If there is already a value in the box they can select it or click on the create new entry in the dropdown and enter the value you want. 
L0037 
L0038 [[image df_pagepath.png]]
L0039 
L0040 **After entering the value you __must__ press Enter.** That will then add the value you have selected or entered and open the next box. There is no limit to the number of these boxes (and the pages they create)  that you can have.  However, you can use the **max-level** property to set the maximum number of levels that can be created in the pagepath tree.
L0041 [[code]]
L0042  origin:
L0043    label: Origin
L0044    type: pagepath
L0045    category: band-origin
L0046    max-level: 4
L0047 [[/code]]
L0048 
L0049 
L0050 In the layout of your page, above the @@====@@ selector, you use @@%%form_data{origin}%%@@ and the lowest value in the pagepath list of values will be displayed. So if you have Europe->UK->London, London will be displayed.
L0051 
L0052 The pages that the pagepath creates can list each of the bands who have that value. To do this, create a live template page containing @@[[module Backlinks]]@@.
L0053 
L0054 
L0055 A site dedicated to examples of the pagepath concept is at *http://pagepath.wikidot.com/
```
