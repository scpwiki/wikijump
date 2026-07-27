# Block Formatting Elements syntax

- Feature ID: `syntax-block-formatting-elements`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented block formatting elements syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:block-formatting-elements/source.wikidot.txt:1` through line 107 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:block-formatting-elements (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:block-formatting-elements/source.wikidot.txt:1` through line 107  
SHA-256 of complete source file: `745a8a49e4eb72d030167f0dcb7a90fda0f30e342f8dd606cd36696bb850d65d`

```wikidot
L0001 ++ [[# left]] Left, right, centered and justified
L0002 
L0003 To apply horizontal alignment to a block of text use:
L0004 
L0005 || {{@@[[<]]@@ _
L0006 ... _
L0007 @@[[/<]]@@}} || align left ||
L0008 || {{@@[[>]]@@ _
L0009 ... _
L0010 @@[[/>]]@@}} || align right ||
L0011 || {{@@[[=]]@@ _
L0012 ... _
L0013 @@[[/=]]@@}} || align center ||
L0014 || {{@@[[==]]@@ _
L0015 ... _
L0016 @@[[/==]]@@}} || align justified ||
L0017 
L0018 E.g.
L0019 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0020 [[code]]
L0021 [[=]]
L0022 Lorem ipsum dolor sit amet, consectetuer adipiscing elit.\
L0023 Aenean a libero. Vestibulum adipiscing, felis ac faucibus \
L0024 imperdiet, erat lacus accumsan neque, vitae nonummy lorem \
L0025 pede ac elit.
L0026 
L0027 Maecenas in urna. Curabitur hendrerit risus vitae ligula.
L0028 [[/=]]
L0029 [[/code]]
L0030 [[/div]]
L0031 
L0032 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0033 [[=]]
L0034 Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean a libero. Vestibulum adipiscing, felis ac faucibus imperdiet, erat lacus accumsan neque, vitae nonummy lorem pede ac elit.
L0035 
L0036 Maecenas in urna. Curabitur hendrerit risus vitae ligula.
L0037 [[/=]]
L0038 [[/div]]
L0039 ~~~~~~~~~
L0040 
L0041 To center a single line use {{=}} at the beginning:
L0042 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0043 [[code]]
L0044 = Centered line
L0045 [[/code]]
L0046 [[/div]]
L0047 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0048 = Centered line
L0049 [[/div]]
L0050 ~~~~~~~~~~
L0051 
L0052 **Note:** The block formatting tags must be on their own line with nothing after them, not even a space. For example, @@[[=]]@@ and @@[[/=]]@@ must be immediately followed by the return character (press Enter).
L0053 
L0054 ++ [[# custom]]Custom //div// blocks
L0055 
L0056 To improve the layout you can use {{@@[[div]] ... [[/div]]@@}} elements which transform to html {{@< <div> ... </div> >@}} blocks.
L0057 Allowed attributes are: {{id}}, {{class}}, {{style}}, {{data-}} only but this should be more than enough to create desired layout. SPAN elements also allow {{class}}, {{style}} and {{data-}} attributes.
L0058 
L0059 {{@@[[div]]@@}} blocks can be nested.  Put the @@[[div]]@@ and @@[[/div]]@@ tags on their own lines or the parser will not recognize them.
L0060 
L0061 Below is an example how to create a 2-column layout using div block:
L0062 [[code]]
L0063 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0064 left column left column left column left column left column
L0065 left column left column left column left column left column
L0066 [[/div]]
L0067 [[div style="float:left; width: 45%; padding: 0 2%"]]
L0068 right column right column right column right column right column
L0069 right column right column right column right column right column
L0070 [[/div]]
L0071 
L0072 ~~~~
L0073 [[/code]]
L0074 
L0075 [[div style="float:left; width: 40%; padding: 0 4%"]]
L0076 left column left column left column left column left column left column left column left column left column left column
L0077 [[/div]]
L0078 [[div style="float:left; width: 40%; padding: 0 4%"]]
L0079 right column right column right column right column right column right column right column right column right column right column
L0080 [[/div]]
L0081 
L0082 ~~~~
L0083 
L0084 The {{~~~~}} element is used to clear floats and translates more or less to {{<div style="clear:both"></div>}}).
L0085 
L0086 Custom {{@@[[div]]@@}} blocks can be used to create very advanced page layouts.
L0087 
L0088 Adding underscore to **div** element **@@[[div_ ]]@@** will truncate whitespaces around it which prevents creation of random [[[doc-wiki-syntax:paragraphs-and-newline | new lines and paragraphs]]]. It's simplifices creation of complex HTML syntax like [[[http://getbootstrap.com/components/ | Bootstrap components]]]
L0089 
L0090 [[div class="alert alert-info"]]
L0091 You can use user-defined {{ID}} arguments in custom DIVs, which is extremely useful building sites using [http://getbootstrap.com Bootstrap]. Please note that every user-defined {{ID}} will have a {{"u-"}} prefix added in the output HTML for the security reasons.
L0092 
L0093 To make your source more readable, you can add the {{"u-"}} prefix yourself. For example, these 2 bits of wiki syntax will output the same HTML:
L0094 ----
L0095 **{{"u-"}} prefix will be added to {{myCarousel}} automatically when the page is saved**
L0096 [[code]]
L0097 [[div id="myCarousel" class="carousel slide" data-interval="3000" data-ride="carousel"]]
L0098 [[/code]]
L0099 **{{"u-"}} prefix will not be added to since it already exists**
L0100 [[code]]
L0101 [[div id="u-myCarousel" class="carousel slide" data-interval="3000" data-ride="carousel"]]
L0102 [[/code]]
L0103 **HTML output from both examples**
L0104 [[code type="html'']]
L0105 <div id="u-myCarousel" class="carousel slide" data-interval="3000" data-ride="carousel">
L0106 [[/code]]
L0107 [[/div]]
```
