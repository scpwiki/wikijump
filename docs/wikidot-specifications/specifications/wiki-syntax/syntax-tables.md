# Tables syntax

- Feature ID: `syntax-tables`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented tables syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:tables/source.wikidot.txt:1` through line 139 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:tables (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:tables/source.wikidot.txt:1` through line 139  
SHA-256 of complete source file: `22c1b340e4c8cb677af077add3ca9530af33c0d9f4c1ee182d6860dfaff6b21f`

```wikidot
L0001 ++ [[# simple]] Simple tables
L0002 
L0003 You can create simple tables using pairs of vertical bars:
L0004 
L0005 [[div style="float:left; width: 45%; margin: 0 2%"]]
L0006 [[code]]
L0007 ||~ head 1 ||~ head 2 ||~ head 3 ||
L0008 || cell 1 || cell 2 || cell 3 ||
L0009 |||| long cell 4 || cell 5 ||
L0010 ||cell 6 |||| long cell 7 ||
L0011 |||||| looong cell 8||
L0012 [[/code]]
L0013 [[/div]]
L0014 
L0015 [[div style="float:left; width: 45%; margin: 0 2%"]]
L0016 ||~ head 1 ||~ head 2 ||~ head 3 ||
L0017 || cell 1 || cell 2 || cell 3 ||
L0018 |||| long cell 4 || cell 5 ||
L0019 ||cell 6 |||| long cell 7 ||
L0020 |||||| looong cell 8||
L0021 [[/div]]
L0022 ~~~~~~~~~~~~~~
L0023 
L0024 [[code]]
L0025 || lines must start and end || with double vertical bars || nothing ||
L0026 || cells are separated by || double vertical bars || nothing ||
L0027 |||| you can span multiple columns by || starting each cell ||
L0028 || with extra cell |||| separators ||
L0029 |||||| but perhaps an example is _
L0030 the easiest way to see ||
L0031 [[/code]]
L0032 
L0033 || lines must start and end || with double vertical bars || nothing ||
L0034 || cells are separated by || double vertical bars || nothing ||
L0035 |||| you can span multiple columns by || starting each cell ||
L0036 || with extra cell |||| separators ||
L0037 |||||| but perhaps an example is _
L0038 the easiest way to see ||
L0039 
L0040 For a new line inside the table cell use _ (underscore) at the end of the line (see the example above).
L0041 
L0042 ++ [[# advanced]]Advanced (custom) tables
L0043 
L0044 To create more advanced tables, special tags can be used that can accept {{class}} and {{style}} attributes for managing appearance. To create an advanced table use the following syntax:
L0045 
L0046 [[code]]
L0047 [[table]]
L0048 [[row]]
L0049 [[hcell style="border: 1px solid silver; background-color: yellow;"]]
L0050 header cell 0.0
L0051 [[/hcell]]
L0052 [[hcell style="border: 1px solid silver"]]
L0053 header cell 0.1
L0054 [[/hcell]]
L0055 [[hcell style="border: 1px solid silver" ]]
L0056 header cell 0.2
L0057 [[/hcell]]
L0058 [[/row]]
L0059 [[row]]
L0060 [[cell style="border: 1px solid silver" colspan="2"]]
L0061 cell 1.0
L0062 [[/cell]]
L0063 [[cell style="border: 1px solid silver; background-color: yellow;"]]
L0064 cell 1.2
L0065 [[/cell]]
L0066 [[/row]]
L0067 [[row]]
L0068 [[cell style="border: 1px solid silver" rowspan="2"]]
L0069 cell 2.0
L0070 [[/cell]]
L0071 [[cell style="border: 1px solid silver"]]
L0072 cell 2.1
L0073 [[/cell]]
L0074 [[cell style="border: 1px solid silver"]]
L0075 cell 2.2
L0076 [[/cell]]
L0077 [[/row]]
L0078 [[row]]
L0079 [[cell style="border: 1px solid silver"]]
L0080 cell 3.1
L0081 [[/cell]]
L0082 [[cell style="border: 1px solid silver"]]
L0083 cell 3.2
L0084 [[/cell]]
L0085 [[/row]]
L0086 [[/table]]
L0087 [[/code]]
L0088 
L0089 transforms to...
L0090 
L0091 [[table]]
L0092 [[row]]
L0093 [[hcell style="border: 1px solid silver; background-color: yellow;"]]
L0094 header cell 0.0
L0095 [[/hcell]]
L0096 [[hcell style="border: 1px solid silver"]]
L0097 header cell 0.1
L0098 [[/hcell]]
L0099 [[hcell style="border: 1px solid silver" ]]
L0100 header cell 0.2
L0101 [[/hcell]]
L0102 [[/row]]
L0103 [[row]]
L0104 [[cell style="border: 1px solid silver" colspan="2"]]
L0105 cell 1.0
L0106 [[/cell]]
L0107 [[cell style="border: 1px solid silver; background-color: yellow;"]]
L0108 cell 1.2
L0109 [[/cell]]
L0110 [[/row]]
L0111 [[row]]
L0112 [[cell style="border: 1px solid silver" rowspan="2"]]
L0113 cell 2.0
L0114 [[/cell]]
L0115 [[cell style="border: 1px solid silver"]]
L0116 cell 2.1
L0117 [[/cell]]
L0118 [[cell style="border: 1px solid silver"]]
L0119 cell 2.2
L0120 [[/cell]]
L0121 [[/row]]
L0122 [[row]]
L0123 [[cell style="border: 1px solid silver"]]
L0124 cell 3.1
L0125 [[/cell]]
L0126 [[cell style="border: 1px solid silver"]]
L0127 cell 3.2
L0128 [[/cell]]
L0129 [[/row]]
L0130 [[/table]]
L0131 
L0132 
L0133 Each of elements @@[[table]]@@, @@[[row]]@@, @@[[cell]]@@ and @@[[hcell]]@@ can accept attributes {{style}} and {{class}} and they are transformed to (X)HTML tags: {{<table>}}, {{<tr>}} and {{<td>}}. Cells also accept **colspan** and **rowspan** variables.
L0134 
L0135 If you wish to remove the spacing between cells in the above example, change the first line to {{@@[[table style="border-collapse:collapse;"]]@@}}.
L0136 
L0137 An example of using tables for page layout can be found on our Snippets Wiki at: http://snippets.wikidot.com/code:layout-with-tables .
L0138 
L0139 Tables can be nested.
```
