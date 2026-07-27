# Universal Escaping syntax

- Feature ID: `syntax-universal-escaping`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented universal escaping syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:universal-escaping/source.wikidot.txt:1` through line 31 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:universal-escaping (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:universal-escaping/source.wikidot.txt:1` through line 31  
SHA-256 of complete source file: `2d31ae3bd9420e5e79cb0983840fe242db1fe304836d33b461de01f174ad16b5`

```wikidot
L0001 If you want to put arbitrary characters or HTML entities (including Unicode entities) into your text, use @<@&lt;>@ ... @<&gt;@>@.  Inside this sequence, convert each "&" to "&amp;", each "<" to "&lt;" and each ">" to "&gt;".
L0002 
L0003 The escape sequence will decode HTML entities like &lt; including:
L0004 
L0005 * entities such as &copy; (@<&copy;>@)
L0006 * numeric entities like &#252; (@<&#252;>@)
L0007 * Unicode entities like &#8212; (@<&#8212;>@) or &auml; (@<&auml;>@)
L0008 
L0009 ++ Live example
L0010 
L0011 [[code]]
L0012 HTML entities: @<U umlaut: &#252;>@
L0013 @<[[code]]>@
L0014 @<Hello world @@ !!!!>@
L0015 @<Something **not** bold>@
L0016 @<[[module ListPages]]>@
L0017 @<Copyright sign: &copy;>@
L0018 @<[[/code]]>@
L0019 Or, @<@&lt;>@ and @<&gt;@>@
L0020 [[/code]]
L0021 
L0022 Which gives:
L0023 
L0024 HTML entities: @<U umlaut: &#252;>@
L0025 @<[[code]]>@
L0026 @<Hello world @@ !!!!>@
L0027 @<Something **not** bold>@
L0028 @<[[module ListPages]]>@
L0029 @<Copyright sign: &copy;>@
L0030 @<[[/code]]>@
L0031 Or, @<@&lt;>@ and @<&gt;@>@
```
