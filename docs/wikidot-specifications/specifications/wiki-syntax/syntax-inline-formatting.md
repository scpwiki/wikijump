# Inline Formatting syntax

- Feature ID: `syntax-inline-formatting`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented inline formatting syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:inline-formatting/source.wikidot.txt:1` through line 33 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:inline-formatting (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:inline-formatting/source.wikidot.txt:1` through line 33  
SHA-256 of complete source file: `5df2e8b6488be9c5bdd5d2df6ff9bc36fa59541fbd89f9893489ff4f5dfbcc30`

```wikidot
L0001 ||~ what you type ||~ what you get ||
L0002 || {{@@//italic text//@@}} || //italic text// ||
L0003 || {{@@**bold text**@@}} || **bold text** ||
L0004 || {{@@//**italic and bold**//@@}} || //**italic and bold**// ||
L0005 || {{@@__underline text__@@}} || __underline text__ ||
L0006 || {{@@--strikethrough text--@@}} || --strikethrough text-- ||
L0007 || {{@@{{teletype (monospaced) text}}@@}} || {{teletype (monospaced) text}} ||
L0008 || {{@@normal^^superscript^^@@}} || normal^^superscript^^ ||
L0009 || {{@@normal,,subscript,,@@}} || normal,,subscript,, ||
L0010 || {{@@[!-- invisible comment --]@@}} || [!-- invisible comment --] ||
L0011 || {{@@[[span style="color:red"]]custom //span// element[[/span]]@@}} || [[span style="color:red"]]custom //span// element[[/span]] ||
L0012 || {{@@##blue|predefined## or ##44FF88|custom-code## color@@}} || ##blue|predefined## or ##229966|custom-code## color ||
L0013 
L0014 Adding underscore to **span** element **@@[[span_ ]]@@** will truncate whitespaces around it which prevents creation of random [[[doc-wiki-syntax:paragraphs-and-newline | new lines and paragraphs]]]. It's simplifices creation of complex HTML syntax like [[[http://getbootstrap.com/components/ | Bootstrap components]]]
L0015 
L0016 [[div class="alert alert-info"]]
L0017 You can use user-defined {{ID}} arguments in **@@[[span]]...[[/span]]@@** tags, which is extremely useful building sites using [http://getbootstrap.com Bootstrap]. Please note that every user-defined {{ID}} will have a {{"u-"}} prefix added in the output HTML for the security reasons.
L0018 
L0019 To make your source more readable, you can add the {{"u-"}} prefix yourself. For example, these 2 bits of wiki syntax will output the same HTML:
L0020 ----
L0021 **{{"u-"}} prefix will be added to {{mySpan}} automatically when the page is saved**
L0022 [[code]]
L0023 [[span id="mySpan"]]My span element[[/span]]
L0024 [[/code]]
L0025 **{{"u-"}} prefix will not be added to since it already exists**
L0026 [[code]]
L0027 [[span id="u-mySpan"]]My span element[[/span]]
L0028 [[/code]]
L0029 **HTML output from both examples**
L0030 [[code type="html'']]
L0031 <span id="u-mySpan">My span element</span>
L0032 [[/code]]
L0033 [[/div]]
```
