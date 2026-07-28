# The 'static' field type

- Feature ID: `data-forms-static-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'static' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:static-field/source.wikidot.txt:1` through line 73 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:static-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:static-field/source.wikidot.txt:1` through line 73  
SHA-256 of complete source file: `6714a78ec92b4718453dd0308f93438d7fb17bfabafe4cb7e968157be767605b`

```wikidot
L0001 This shows non-editable text and lets the form designer add text and formatting to the form. Static fields are not stored in the page. Static fields get their value from the 'value' property.
L0002 
L0003 [[code]]
L0004 [[form]]
L0005 fields:
L0006   version:
L0007     type: static
L0008     value: 'Non-storable field with with **bold**, //strike// and __underline__.'
L0009 [[/form]]
L0010 [[/code]]
L0011 
L0012 The specific properties you can use on a static field:
L0013 
L0014 * **value**: sets the value of the field
L0015 
L0016 The static field can use most wiki syntax and you can easily add line breaks by using the pipe character (**|**) to start a block of text for the value property. For example, the static field below contains the source code from the **Inline Formatting** documentation section. Formatting your value property this way is easy since all of the code is parsed as-is and you don't have to worry about escaping single quotes or adding line break code.:
L0017 [[code]]
L0018 [[form]]
L0019 fields:
L0020   date-1:
L0021     type: date
L0022     hint: 'mm/dd/yyyy'
L0023     label: 'Date 1'
L0024     options:
L0025       showOn: button
L0026       showAnim: slideDown
L0027       duration: 4000
L0028   header-1:
L0029     type: static
L0030     label: 'Inline Formatting Docs'
L0031     value: |
L0032              //italic text//    italic text
L0033              ||~ what you type ||~ what you get ||
L0034              || {{@@//italic text//@@}} || //italic text// ||
L0035              || {{@@**bold text**@@}} || **bold text** ||
L0036              || {{@@//**italic and bold**//@@}} || //**italic and bold**// ||
L0037              || {{@@__underline text__@@}} || __underline text__ ||
L0038              || {{@@--strikethrough text--@@}} || --strikethrough text-- ||
L0039              || {{@@{{teletype (monospaced) text}}@@}} || {{teletype (monospaced) text}} ||
L0040              || {{@@normal^^superscript^^@@}} || normal^^superscript^^ ||
L0041              || {{@@normal,,subscript,,@@}} || normal,,subscript,, ||
L0042              || {{@@[!-- invisible comment --]@@}} || [!-- invisible comment --] ||
L0043              || {{@@[[span style="color:red"]]custom //span// element[[/span]]@@}} || [[span style="color:red"]]custom //span// element[[/span]] ||
L0044              || {{@@##blue|predefined## or ##44FF88|custom-code## color@@}} || ##blue|predefined## or ##229966|custom-code## color ||
L0045 
L0046              [[div class="alert alert-info"]]
L0047              You can use user-defined {{ID}} arguments in **@@[[span]]...[[/span]]@@** tags, which is extremely useful building sites using [http://getbootstrap.com Bootstrap]. Please note that every user-defined {{ID}} will have a {{"u-"}} prefix added in the output HTML for the security reasons.
L0048              [[/div]]
L0049 [[/form]]
L0050 [[/code]] 
L0051 
L0052 The same code can be entered as shown below by wrapping the whole string in double quotes and then using **\n** to indicate where line break should fall. It will render the same result. You can see that the above code is much easier to read and to write.
L0053 [[code]]
L0054 [[form]]
L0055 fields:
L0056   date-1:
L0057     type: date
L0058     hint: mm/dd/yyyy
L0059     label: 'Date 1'
L0060     options:
L0061       showOn: button
L0062       showAnim: slideDown
L0063       duration: 4000
L0064   header-1:
L0065     type: static
L0066     label: 'Inline Formatting Docs'
L0067     value: "//italic text//    italic text\n||~ what you type ||~ what you get ||\n|| {{@@//italic text//@@}} || //italic text// ||\n|| {{@@**bold text**@@}} || **bold text** ||\n|| {{@@//**italic and bold**//@@}} || //**italic and bold**// ||\n|| {{@@__underline text__@@}} || __underline text__ ||\n|| {{@@--strikethrough text--@@}} || --strikethrough text-- ||\n|| {{@@{{teletype (monospaced) text}}@@}} || {{teletype (monospaced) text}} ||\n|| {{@@normal^^superscript^^@@}} || normal^^superscript^^ ||\n|| {{@@normal,,subscript,,@@}} || normal,,subscript,, ||\n|| {{@@[!-- invisible comment --]@@}} || [!-- invisible comment --] ||\n|| {{@@[[span style=\"color:'red\"]]custom //span// element[[/span]]@@}} || [[span style=\"color:red\"]]custom //span// element[[/span]] ||'\n|| {{@@##blue|predefined## or ##44FF88|custom-code## color@@}} || ##blue|predefined## or ##229966|custom-code## color ||\n\n[[div class=\"alert alert-info\"]]\nYou can use user-defined {{ID}} arguments in **@@[[span]]...[[/span]]@@** tags, which is extremely useful building sites using [http://getbootstrap.com Bootstrap]. Please note that every user-defined {{ID}} will have a {{\"u-\"}} prefix added in the output HTML for the security reasons.\n[[/div]]"
L0068 [[/form]]
L0069 [[/code]]
L0070 
L0071 [[div class="alert alert-info"]]
L0072 Wikidot super guru, Kenneth Tsang ([[user tsangk]]), created a great [*http://convert.wikidot.com/ Wikidot Data Form Fixer] tool that you can use to check the syntax and formatting of your data forms. It's highly recommend to make sure your code is 100% compliant with the formatting rules.
L0073 [[/div]]
```
