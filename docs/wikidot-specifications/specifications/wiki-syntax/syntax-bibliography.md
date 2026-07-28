# Bibliography syntax

- Feature ID: `syntax-bibliography`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented bibliography syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:bibliography/source.wikidot.txt:1` through line 21 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:bibliography (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:bibliography/source.wikidot.txt:1` through line 21  
SHA-256 of complete source file: `84b95fe522a365119b1fc9562eca51d40cc4bf736abcaa9157bbfe66730e011c`

```wikidot
L0001 The bibliography block is defined by {{@@[[bibliography]]...[[/bibliography]]@@}}. Each bibliography item has the form:
L0002 {{@@label : full reference@@}}
L0003 To cite a bibliography entry use {{@@((bibcite@@ //label//))}}.
L0004 
L0005 [[code]]
L0006 The first pulsar was observed by J. Bell and A. Hewish [((bibcite bell))]. Another reference [see ((bibcite guy))].
L0007 
L0008 [[bibliography]]
L0009 : bell : Bell, J.; Hewish, A.; Pilkington, J. D. H.; Scott, P. F.; and Collins, R. A. //Observation of a Rapidly Pulsating Radio Source.// Nature 217, 709, 1968.
L0010 : guy : Guy, R. K. //Modular Difference Sets and Error Correcting Codes.// §C10 in Unsolved Problems in Number Theory, 2nd ed. New York: Springer-Verlag, pp. 118-121, 1994.
L0011 [[/bibliography]]
L0012 [[/code]]
L0013 
L0014 The first pulsar was observed by J. Bell and A. Hewish [((bibcite bell))]. Another reference [see ((bibcite guy))].
L0015 
L0016 [[bibliography]]
L0017 : bell : Bell, J.; Hewish, A.; Pilkington, J. D. H.; Scott, P. F.; and Collins, R. A. //Observation of a Rapidly Pulsating Radio Source.// Nature 217, 709, 1968.
L0018 : guy : Guy, R. K. //Modular Difference Sets and Error Correcting Codes.// §C10 in Unsolved Problems in Number Theory, 2nd ed. New York: Springer-Verlag, pp. 118-121, 1994.
L0019 [[/bibliography]]
L0020 
L0021 If you are not satisfied with the default title ("Bibliography") you can force your own title by using {{@@[[bibliography title="Custom title"]]@@}} or even do not use title at all ({{title=""}}).
```
