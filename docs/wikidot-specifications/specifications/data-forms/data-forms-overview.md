# Data Forms

- Feature ID: `data-forms-overview`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Support structured page data defined by category templates and exposed through Wikidot create, edit, display, and query flows.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:reference/source.wikidot.txt:1` through line 17 (supporting)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:start/source.wikidot.txt:1` through line 41 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:data-forms/source.wikidot.txt:1` through line 1 (redirect)

## Documentation-derived behavioral evidence

### doc-data-forms:reference (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:reference/source.wikidot.txt:1` through line 17  
SHA-256 of complete source file: `65398daea9d8032f5af154267cb52b066f8d777fa854f456c3d93316ecb6cb9f`

```wikidot
L0001 The form definition is made in [http://yaml.org YAML], which is a simple structured markup language.  A //_template// may have a single form.  The form starts and ends with @@[[form]]@@ and @@[[/form]]@@ as for code blocks.  Within those tags, we describe the form using YAML:
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:                           #  This is always required at the start
L0006   name-of-the-field:              #  Use a valid YAML name (i.e not starting with a number)
L0007     label: Label                  #  This is what the user sees when using the form
L0008     type: type-of-field           #  The field types
L0009     property: value...            #  Depending on the field type
L0010 [[/form]]
L0011 [[/code]]
L0012 
L0013 The default field type is 'text', unless you specify one or more values, in which case it defaults to 'select'.
L0014 
L0015 [[note]]
L0016 **Always start name of the field form with a letter. Field names starting with a digit or some other character are invalid. In case of special YAML symbols like {{true}}, {{false}}, {{yes}}, {{no}}, you may need to surround those with simple quote signs like this: "yes".**
L0017 [[/note]]
```

### doc-data-forms:start (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:start/source.wikidot.txt:1` through line 41  
SHA-256 of complete source file: `90776aa10aa716f4c5dd0888143431d9b3a2c7fda14bc3ac189b084049cf4db6`

```wikidot
L0001 [[module css]]
L0002 #toc{  width: 300px; }
L0003 pre { white-space:pre-wrap; }
L0004 [[/module]]
L0005 
L0006 [[image df_dataform.jpg]]
L0007 
L0008 Wikidot Data Forms is a very powerful feature that makes it possible to build everything from simple applications in your wikidot sites to a complete content management system (CMS) across your entire site.  
L0009 
L0010 A normal wiki page holds unstructured text.  A wiki page with a data form holds structured data in "fields", the same as a database.  In many cases structured data in a data form is easier for your users to edit, to understand and to work with.
L0011 
L0012 ------
L0013 
L0014 ++ Some uses for data forms
L0015 
L0016 Some of the uses where data forms might work better than simple wiki pages are:
L0017 
L0018 * I'm collecting references for my thesis, and for each reference I want to record the title, author, ISBN, date of issue, publisher, and language.  If I use a data form with one field for each piece of data, I can easily produce reference lists in any format.
L0019 
L0020 * I'm organizing my club membership and for each member I want a page with their name, email address and so on.  By using a data form I can extract fields like the email address to send everyone a newsletter.
L0021 
L0022 * I'm cataloging my video game collection and using a data form means I can search on games by console, by publisher, by genre and so on.
L0023 
L0024 * I want my members to enter information about software, but I want to control what they enter by using lists they select from.
L0025 
L0026 * I want users of my site to be able to easily upload images and videos at the same time that they create a page.
L0027 
L0028 * I want to build a complete site where the user doesn't need to know any Wikidot syntax but can just fill in forms and press Save.
L0029 
L0030 ------
L0031 
L0032 ++ Live demo
L0033 
L0034 * A live demo is available to show the features of data forms that we have described in this documentation. The permissions have been relaxed so you can try out the form:
L0035 
L0036  * main page for creating new pages in the //band// category and for listing bands: *http://vineyard.wikidot.com/bands:main
L0037  * example page at *http://vineyard.wikidot.com/band:queen
L0038  * live template at *http://vineyard.wikidot.com/band:_template
L0039 
L0040 * [http://pagepath.wikidot.com/ pagepath.wikidot.com] shows examples of the  //pagepath// concept. 
L0041 * There is also a pagepath example using the band example at *http://vineyard.wikidot.com/bands
```

### doc:data-forms (redirect)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:data-forms/source.wikidot.txt:1` through line 1  
SHA-256 of complete source file: `8e845b21ae43ae3683dd14764d31a2df10014e4d77bb99f51883464770e7a3fc`

```wikidot
L0001 [[module Redirect destination="doc-data-forms:start"]]
```
