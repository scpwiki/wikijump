# How to create a new data form

- Feature ID: `data-forms-howto`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “How to create a new data form”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:howto/source.wikidot.txt:1` through line 56 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:howto (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:howto/source.wikidot.txt:1` through line 56  
SHA-256 of complete source file: `caadd31a8d462685423add63041872b3cd23b124d9d9c297ba9d0416a053b4fe`

```wikidot
L0001 Wikidot stores normal pages in categories and it is exactly the same when you use data forms. Each data form page is one page in a specific category. A category can have only one data form and that data form structure applies to all pages in that category, so you cannot mix data form pages and normal wiki pages in the same category.
L0002 
L0003 To create a new data form you need to do the following:
L0004 
L0005 1) create a live template page for the category the form will be in. For example if your category is //band//, the live template page must be called //band:_template//.
L0006 
L0007 2) add a @@[[form]] ..[[/form]]@@ section then your fields. The different types of fields you can have (text, select, checkbox, file, wiki, static, hidden and password are described in the reference section at the bottom of this page.
L0008 
L0009 Please note that the indentation shown in the example below is important because if the different rows are not indented correctly the fields will not display. Your structure should look like the example below, but note that you don't have to enter a field type and a width; if you don't enter a field type it will default to a text field type. The width is also not mandatory.
L0010 
L0011 Please note that for all fields you must have a space between the colon and the value, for example **label: Music type** is correct, but if you enter **label:Music type** you will get n error message when you try to save the page.
L0012 
L0013 
L0014 [[code]]
L0015 [[form]]
L0016 fields:
L0017   type:
L0018     label: Music type
L0019     type: select
L0020     values:
L0021       0: Classical
L0022       1: Country
L0023       2: Folk
L0024       3: Indie
L0025       4: Jazz
L0026       5: Pop
L0027       6: Rock
L0028     default: 6
L0029   bandimage:
L0030     label: Image
L0031     type: file
L0032   bandwebsite:
L0033     label: Band website
L0034     type: url
L0035   current:
L0036     label: Currently Recording
L0037     type: select
L0038     values:
L0039       0: "Yes"
L0040       1: "No"
L0041     default: 0
L0042 [[/form]]
L0043 [[/code]]
L0044 
L0045 After you define a @@[[form]] ..[[/form]]@@ structure like the one above, when you edit add or edit any page in the category it shows the form instead of the normal page editor.
L0046 
L0047 ++ Checking for errors
L0048 Wikidot used to be relaxed about whether there were spaces after the colon, but now a more strict version of the code is used which will give you an error if you have built your data form with incorrect spaces. However, there is an app developed by one of our gurus, [[*user tsangk]] to test whether your data form has been built correctly and has the correct spacing. The app is at *http://community.wikidot.com/app:convert. You just copy and paste your whole page into the app and it will convert the data form to the correct structure if it finds errors.
L0049 
L0050 ------
L0051 
L0052 ++ Setting up your Site Manager
L0053 
L0054 You can configure category permissions for a category with a data form exactly as for normal categories so that, for example, only the author of a page can edit it.  
L0055 
L0056 It is sometimes a very good idea to __autonumber__ the category containing the data form. This will remove the risk of duplicate page names. This is setup in the //site manager > autonumbering of pages//.
```
