# Foldable List syntax

- Feature ID: `syntax-foldable-list`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented foldable list syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:foldable-list/source.wikidot.txt:1` through line 27 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:foldable-list (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:foldable-list/source.wikidot.txt:1` through line 27  
SHA-256 of complete source file: `e16ef1db3742957f3733d9b00bf626d22d697f349a00048e0a3834f350976a76`

```wikidot
L0001 The [*http://snippets.wikidot.com/code:foldable-list Foldable List] container is a special class that can be used in a @@[[div]]@@. It is useful for creating a navigation menu that folds and unfolds to expose different levels of a list. The following example shows how you can create 3 levels of nesting.
L0002 [[code]]
L0003 [[div class="foldable-list-container"]]
L0004 * Links
L0005  * Wikidot
L0006   * [*http://www.wikidot.com/doc Documentation]
L0007   * [*http://www.wikidot.com/doc:wiki-syntax wiki-syntax]
L0008   * [*http://community.wikidot.com/howto:howto-list How-To's]
L0009  * Search Engines
L0010   * [*http://www.google.com Google]
L0011   * [*http://www.yahoo.com Yahoo]
L0012 * Main Category 1
L0013  * [# Main 1 - Sub 1]
L0014  * [# Main 1 - Sub 2]
L0015 * Main Category 2
L0016  * [# Main 2 - Sub 1]
L0017  * [# Main 2 - Sub 2]
L0018  * [# Main 2 - Sub 3]
L0019 * Main Category 3
L0020  * [# Main 3 - Sub 1]
L0021 [[/div]]
L0022 [[/code]]
L0023 
L0024 [[div class="alert alert-info"]]
L0025 **Update note:** 
L0026 * it can be used anywhere (no longer associated with {{side bar}})
L0027 [[/div]]
```
