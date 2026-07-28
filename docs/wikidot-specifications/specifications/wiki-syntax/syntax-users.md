# Users syntax

- Feature ID: `syntax-users`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented users syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:users/source.wikidot.txt:1` through line 5 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:users (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:users/source.wikidot.txt:1` through line 5  
SHA-256 of complete source file: `1170d8628c4fabf4c28e0d866f36f93aa43c4afe5e643b84987d4bf21e438a5b`

```wikidot
L0001 ||~ what you type ||~ what you get ||~ comments||
L0002 || {{@@[[user@@ //user-name//]]}} _
L0003  e.g. {{@@[[user michal frackowiak]]@@}} ||  [[user michal frackowiak]] || user info (no buddy icon)||
L0004 || {{@@[[*user@@ //user-name//]]}} _
L0005  e.g. {{@@[[*user michal frackowiak]]@@}} ||  [[*user michal frackowiak]] || user info (with buddy icon)||
```
