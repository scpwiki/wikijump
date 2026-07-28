# Lists syntax

- Feature ID: `syntax-lists`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented lists syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:lists/source.wikidot.txt:1` through line 105 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:lists (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:lists/source.wikidot.txt:1` through line 105  
SHA-256 of complete source file: `bc4689e649adaeda621d552ad4c2cef405bf5b4fa19843df24ce5dfd657fd29a`

```wikidot
L0001 ++ [[# bulleted]] Bulleted Lists
L0002 
L0003 Make a list element by starting a line with an asterisk. To increase the indent put extra spaces
L0004 before the asterisk.
L0005 
L0006 [[code]]
L0007 * Bullet 1
L0008 * Bullet 2
L0009  * Bullet 2.1
L0010 [[/code]]
L0011 
L0012 * Bullet 1
L0013 * Bullet 2
L0014  * Bullet 2.1
L0015 
L0016 If you need to put more than one line in the bullet list, please use _ (underscore) at the end of the line you want to break (after one space). Remember not to insert any character after the underscore.
L0017 
L0018 [[code]]
L0019 * Bullet 1 _
L0020  another line
L0021 * Bullet 2
L0022  * Bullet 2.1
L0023 [[/code]]
L0024 
L0025 * Bullet 1 _
L0026  another line
L0027 * Bullet 2
L0028  * Bullet 2.1
L0029 
L0030 ++ [[# numbered]] Numbered Lists
L0031 
L0032 Similarly, you can create numbered lists by starting a paragraph with one or more hashes.
L0033 
L0034 [[code]]
L0035 # Item 1
L0036 # Item 2
L0037  # Item 2.1
L0038 [[/code]]
L0039 
L0040 # Item 1
L0041 # Item 2
L0042  # Item 2.1
L0043 
L0044 If you need to put more than one line in the numbered list, please use _ (underscore) at the end of the line you want to break (after one space). Remember not to insert any character after the underscore.
L0045 
L0046 [[code]]
L0047 # Item 1 _
L0048  another line
L0049 # Item 2
L0050  # Item 2.1
L0051 [[/code]]
L0052 
L0053 # Item 1 _
L0054  another line
L0055 # Item 2
L0056  # Item 2.1
L0057 
L0058 You can mix bulleted lists and number lists.
L0059 
L0060 ++ [[# advanced]] Advanced Lists
L0061 
L0062 You can use @@[[ul]]@@ / @@[[ol]]@@ and @@[[li]]@@ tags to create advanced lists. It's especially useful when using a Boostrap-based theme. Every @@[[ul]]@@ / @@[[ol]]@@ and @@[[li]]@@ can contain //id//, //class//, //data-// and //style// arguments. Lists can be nested.
L0063 
L0064 [[code]]
L0065 [[ul]]
L0066  [[li class="item1" data-toggle="data1"]]Item1[[/li]]
L0067  [[li style="color: red;"]]Item 2
L0068   [[ol]]
L0069     [[li]]Item 2.1[[/li]]
L0070     [[li]]Item 2.2[[/li]]
L0071   [[/ol]]
L0072  [[/li]]
L0073 [[/ul]]
L0074 [[/code]]
L0075 
L0076 [[ul]]
L0077  [[li class="item1" data-toggle="data1"]]Item1[[/li]]
L0078  [[li style="color: red;"]]Item 2
L0079   [[ol]]
L0080     [[li]]Item 2.1[[/li]]
L0081     [[li]]Item 2.2[[/li]]
L0082   [[/ol]]
L0083  [[/li]]
L0084 [[/ul]]
L0085 
L0086 Adding underscore to **ul/ol** element **@@[[ul_/ol_ ]]@@** will truncate whitespaces around it which prevents creation of random [[[doc-wiki-syntax:paragraphs-and-newline | new lines and paragraphs]]]. It's simplifices creation of complex HTML syntax like [[[http://getbootstrap.com/components/ | Bootstrap components]]]
L0087 
L0088 [[div class="alert alert-info"]]
L0089 You can use user-defined {{ID}} arguments in advanced lists, which is extremely useful building sites using [http://getbootstrap.com Bootstrap]. Please note that every user-defined {{ID}} will have a {{"u-"}} prefix added in the output HTML for the security reasons.
L0090 
L0091 To make your source more readable, you can add the {{"u-"}} prefix yourself. For example, these 2 bits of wiki syntax will output the same HTML:
L0092 ----
L0093 **{{"u-"}} prefix will be added to {{myAdvancedList}} automatically when the page is saved**
L0094 [[code]]
L0095 [[ul id="myAdvancedList"]]
L0096 [[/code]]
L0097 **{{"u-"}} prefix will not be added to since it already exists**
L0098 [[code]]
L0099 [[ul id="u-myAdvancedList"]]
L0100 [[/code]]
L0101 **HTML output from both examples**
L0102 [[code type="html'']]
L0103 <ul id="u-myAdvancedList">
L0104 [[/code]]
L0105 [[/div]]
```
