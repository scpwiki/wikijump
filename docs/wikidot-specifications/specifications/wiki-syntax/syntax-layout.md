# Layout elements syntax

- Feature ID: `syntax-layout`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented layout elements syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:layout/source.wikidot.txt:1` through line 42 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:layout (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:layout/source.wikidot.txt:1` through line 42  
SHA-256 of complete source file: `a8416fe878455fc7771d0ef5a1070a8496202ce5f2fbd3ee466e7c13a3360440`

```wikidot
L0001 ++ [[# tab-view]]Tab view
L0002 
L0003 Tab view is a container that creates some clickable tabs that allow to switch between content to show.
L0004 
L0005 **NOTE: TabView breaks TOCs, anchor links and back button**
L0006 * you can't link to anchor inside of a tab
L0007 * TOC won't link properly to any header inside of a tab
L0008 * if you click a link from within a tab and go back, you will be always shown the first tab
L0009 
L0010 To generate a //tabview//, i.e. a set of tabs, the following syntax can be used:
L0011 
L0012 [[code]]
L0013 [[tabview]]
L0014 [[tab Title of Tab No. 1]]
L0015 Content of Tab No. 1.
L0016 [[/tab]]
L0017 [[tab Title of Tab No. 2]]
L0018 Content of Tab No. 2.
L0019 [[/tab]]
L0020 [[tab Title of Tab No. 3]]
L0021 Content of Tab No. 3.
L0022 [[/tab]]
L0023 [[/tabview]]
L0024 [[/code]]
L0025 
L0026 This will produce the following tabset:
L0027 
L0028 [[tabview]]
L0029 [[tab Title of Tab No. 1]]
L0030 Content of Tab No. 1.
L0031 [[/tab]]
L0032 [[tab Title of Tab No. 2]]
L0033 Content of Tab No. 2.
L0034 [[/tab]]
L0035 [[tab Title of Tab No. 3]]
L0036 Content of Tab No. 3.
L0037 [[/tab]]
L0038 [[/tabview]]
L0039 
L0040 Tabs will accept any content, but at the moment it is not possible to nest tabviews.
L0041 
L0042 Another example of {{tabview}} can be found at our Snippets Wiki at http://snippets.wikidot.com/code:tabs
```
