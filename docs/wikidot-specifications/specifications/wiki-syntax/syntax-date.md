# Date syntax

- Feature ID: `syntax-date`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented date syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:date/source.wikidot.txt:1` through line 50 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:date (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:date/source.wikidot.txt:1` through line 50  
SHA-256 of complete source file: `de5d94f336378e4cf3ed0ff7adcfa1b5ef329118b3e5e25eb293e3ac2a0d1c21`

```wikidot
L0001 In several places (forum, private messages, last revision date, etc.) Wikidot pages use dates and timestamps that automatically calculate (either when hovering with the mouse or directly in the text) how long ago this was. Examples are
L0002 * [[date 1237135440 format="%e %b %Y, %H:%M %Z|agohover"]] (move the mouse over the date to see the hovering text) or,
L0003 * [[date 1237135440 format="%c"]]
L0004 
L0005 If you want dates that //you// type on your pages to also automatically show "how long ago'', here is how you can do it.
L0006 
L0007 ++ [[# how]] How it works
L0008 The syntax needed is:
L0009 > {{@@[[date@@ //timestamp// <format="//format//@@<|agohover>">]]@@}}
L0010 
L0011 where
L0012 * < ... > denote optional parameters
L0013 * {{//timestamp//}} is the number of seconds between Jan 1, 1970 and the wanted date. To find this number for a specific date, see [#wizard Code Wizard] below.
L0014 * {{//format//}} is an arbitrary text string that may include //[http://community.wikidot.com/howto:frontforum-date-variable#modifiers %modifiers]//, which are replaced by an actual (part of the) date or time. If not specified, {{//format//}} defaults to "%e''.
L0015 * {{|agohover}} when specified displays a "hovering'' text ("//nn// seconds/minutes/hours/days ago") when the mouse is moved over any part of the displayed {{//format//}} string.
L0016 
L0017 ++ [[# wizard]][[# example]]Code Wizard
L0018 To find out what code you should use on your page for a specific date:
L0019 [[iframe http://community.wikidot.com/howto:date-how-long-ago/code/1 frameborder="0" scrolling="no" width="100%" height="210px"]]
L0020 Then copy/paste the displayed code into your page.
L0021 
L0022 ++ [[# examples]]Examples
L0023 [[table style="border-collapse:collapse;border-top:2px solid;border-bottom:2px solid"]]
L0024 [[row]]
L0025 [[cell style="padding:3px 1em 3px 0"]]//**What you type ...**//[[/cell]][[cell]]//**What you get ...**//[[/cell]]
L0026 [[/row]]
L0027 [[row style="border-top:1px solid"]]
L0028 [[cell style="padding:3px 1em 3px 0"]]{{@@[[date 1216153821]]@@}}[[/cell]]
L0029 [[cell]][[date 1216153821]][[/cell]]
L0030 [[/row]]
L0031 [[row style="border-top:1px solid"]]
L0032 [[cell style="padding:3px 1em 3px 0"]]{{@@[[date 1216153821 format="%d. %m. %Y|agohover"]]@@}}[[/cell]]
L0033 [[cell]][[date 1216153821 format="%d. %m. %Y|agohover"]][[/cell]]
L0034 [[/row]]
L0035 [[row style="border-top:1px solid"]]
L0036 [[cell style="padding:3px 1em 3px 0"]]{{@@[[date 681746400 format="James is %O young"]]@@}}[[/cell]]
L0037 [[cell]][[date 681746400 format="James is %O young"]][[/cell]]
L0038 [[/row]]
L0039 [[row style="border-top:1px solid"]]
L0040 [[cell style="padding:3px 1em 3px 0"]]{{@@+++ Minutes from [[date 1234567890 format="%e %B|agohover"]]@@}}[[/cell]]
L0041 [[cell]]
L0042 +++* Minutes from [[date 1234567890 format="%e %B|agohover"]]
L0043 [[/cell]]
L0044 [[/row]]
L0045 [[/table]]
L0046 
L0047 Note: You can use %O also with the future dates as well.
L0048 
L0049 **Author**
L0050 created by [[*user ErichSteinboeck]]
```
