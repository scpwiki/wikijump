# Selecting & Sorting by Data Form fields

- Feature ID: `data-forms-selecting-and-sorting`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Selecting & Sorting by Data Form fields”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:selecting-and-sorting/source.wikidot.txt:1` through line 92 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:selecting-and-sorting (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:selecting-and-sorting/source.wikidot.txt:1` through line 92  
SHA-256 of complete source file: `ef98d33782843de39360b20c1c124f48d16ae044b91229f8a4c49c3d24aac1b3`

```wikidot
L0001 Using the ListPages module you can select data from a field in the data form and you can also sort by the values within a data form field.
L0002 
L0003 +++ [[# selecting]] Selecting
L0004 
L0005 Add a field to your data form or use an existing. With our band example we have added a field to note whether the band will visit Scotland on their next tour:
L0006 
L0007 [[code]]
L0008  scotland:
L0009   label: Next tour will visit Scotland
L0010   type: select
L0011   values:
L0012      info: No Info
L0013      visit: "Yes"
L0014      novisit: "No"
L0015 [[/code]]
L0016 
L0017 To list those where the value is "Yes" use a ListPages module and add a new parameter starting with an underscore then the fieldname, in this case //_scotland// followed by an = sign and the property of the field you want: //_scotland="visit"//
L0018 
L0019 [[code]]
L0020 [[module ListPages category="band"  _scotland="visit" perPage="10" order="name"  separate="false" prependLine="||~ Band||~ Type ||" appendLine="||||||~ ||"]]
L0021 || %%title_linked%% || %%form_data{type}%% ||
L0022 [[/module]]
L0023 
L0024 [[/code]]
L0025 
L0026 That produces a list of just 2 bands:
L0027 
L0028 [[image df_scotland.png]]
L0029 
L0030 
L0031 You can combine several data form selection fields to narrow down your search. For example if we wanted to just select folk bands that will tour Scotland we would use the //_scotland="visit"// selection criteria and the //_type="2"// selection criteria (because type is the data form feld for the music type, and 2 is the property of the folk value). 
L0032 [[code]]
L0033  type:
L0034    label: Music type
L0035    type: select
L0036    values:
L0037      0: Classical
L0038      1: Country
L0039      2: Folk
L0040      3: Indie
L0041      4: Jazz
L0042      5: Pop
L0043      6: Rock
L0044    default: 6
L0045 [[/code]]
L0046 
L0047 Combining different selection criteria uses the AND operator, so the result must match both of these criteria. The resulting ListPages code would look like this:
L0048 
L0049 [[code]]
L0050 [[module ListPages category="band"  _scotland="visit" _type="2" perPage="10" order="name"  separate="false" prependLine="||~ Band||~ Type ||" appendLine="||||||~ ||"]]
L0051 || %%title_linked%% || %%form_data{type}%% ||
L0052 [[/module]]
L0053 [[/code]]
L0054 
L0055 and the table that is produced is:
L0056 
L0057 [[image df_filter.png]]
L0058 
L0059 You can search for pages where a particular field is empty by using **_field=""**
L0060 
L0061 +++ [[# sorting]] Sorting
L0062 You can also sort by data form field properties. In our band example we have created a field to store the number of albums/CDs released by the band:
L0063 
L0064 [[code]]
L0065  albums:
L0066     label: Albums/CDs released
L0067     type: select
L0068     values:
L0069       "00": 0
L0070       "01": 1
L0071       "02": 2
L0072       "03": 3
L0073       "04": 4
L0074       "05": 5
L0075       "06": 6
L0076       "07": 7
L0077       "08": 8
L0078       "09": 9
L0079 ..
L0080 [[/code]]
L0081 
L0082 To sort the number of albums into descending order, use a Listpages module with the //order=//parameter  followed by an underscore then the name of the field then the //desc// attribute: order="_albums desc"
L0083 
L0084 [[code]]
L0085 [[module ListPages category="band"  perPage="10" order="_albums desc" separate="false" prependLine="||~ Band||~ Albums ||" appendLine="||||||~ ||"]]
L0086 || %%title_linked%% || %%form_data{albums}%% ||
L0087 [[/module]]
L0088 [[/code]]
L0089 
L0090 In order for the sort to work correctly, numbers below 10 must have a property of  01, 02, 03 etc. although the value can still be 1, 2, 3 etc as in the example data form field above. It is the value that is displayed in the ListPages module, as shown below. As (01, 02, 03, ...) are treated as octal numbers you need to enclose them by semicolons ("01", "02", "03", ...) because there is no 08 and 09 in octal and they both will be 0.
L0091 
L0092 [[image df_albums.png]]
```
