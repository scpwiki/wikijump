# Horizontal Rules syntax

- Feature ID: `syntax-horizontal-rules`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented horizontal rules syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:horizontal-rules/source.wikidot.txt:1` through line 1 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:quick-reference/source.wikidot.txt:51` through line 51 (supporting)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:horizontal-rules (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:horizontal-rules/source.wikidot.txt:1` through line 1  
SHA-256 of complete source file: `6ae487b3104aad2d2a78e3174d8376a45a0ecefe150767ffaf8ab76bd610874b`

```wikidot
L0001 Use four dashes or more ({{@@----@@}}) to create a horizontal rule.
```

### doc:quick-reference (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:quick-reference/source.wikidot.txt:51` through line 51  
SHA-256 of complete source file: `df8b7f52d5d9b9770a91747d5b6f5dc28c9d133cb9f989f94380395cd0407234`

```wikidot
L0051 || [/doc-wiki-syntax:horizontal-rules ---- Horizontal line] ||
```
