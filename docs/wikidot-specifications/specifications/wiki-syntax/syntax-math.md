# Math syntax

- Feature ID: `syntax-math`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented math syntax, including every documented form, option, output rule, and limitation.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:math/source.wikidot.txt:1` through line 55 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:math (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:math/source.wikidot.txt:1` through line 55  
SHA-256 of complete source file: `86f162af9817a3373e83c97193b30ebe8c1b05b72294e4a4074b87723a9555d0`

```wikidot
L0001 Wikidot.com uses [http://mathjax.org MathJax] to render beautiful LaTeX equations. For those that know LaTeX syntax using wikidot equations should be straightforward.
L0002 
L0003 ++ [[# equations]] Equations
L0004 
L0005 Simply put the equation between {{@@[[math@@ //label//]] ... @@[[/math]]@@}} block tags (the label is optional). The equation is rendered within LaTex {{@@\begin{equation} ... \end{equation}@@}} environment. Please refer to any LaTeX reference manual for details about syntax.
L0006 
L0007 [[code]]
L0008 [[math label1]]
L0009 \rho _{\rm GJ} = -\sigma (r) \left[ (1 - \eta _{\ast }^2 {\kappa \over {\eta ^3}}) \cos \chi \right.
L0010 + \left. {3\over 2} \theta (\eta) H(\eta)
L0011 \xi \sin \chi \cos \phi \right]
L0012 [[/math]]
L0013 [[/code]]
L0014 
L0015 [[math label1]]
L0016 \rho _{\rm GJ} = -\sigma (r) \left[ (1 - \eta _{\ast }^2 {\kappa \over {\eta ^3}}) \cos \chi \right.
L0017 + \left. {3\over 2} \theta (\eta) H(\eta)
L0018 \xi \sin \chi \cos \phi \right]
L0019 [[/math]]
L0020 
L0021 To refer to a __labeled__ equation simply use {{@@[[eref@@ //label//]]}} to get a raw number or e.g. {{@@Eq. ([[eref@@ //label1//]])}} which gives Eq. ([[eref label1]]).
L0022 
L0023 You can specify the LaTeX environment in 2 ways, either by passing a {{type="<environment>"}} parameter, or using {{@@\begin{<environment>}@@}} and {{@\end{<environment>}@@}} within the equation. E.q. these two are equivalent:
L0024 
L0025 [[code]]
L0026 [[math type="align"]]
L0027 E = mc^2
L0028 [[/math]]
L0029 [[/code]]
L0030 
L0031 [[code]]
L0032 [[math]]
L0033 \begin{align}
L0034 E = mc^2
L0035 \end{align}
L0036 [[/math]]
L0037 [[/code]]
L0038 
L0039 The {{equation}} environment is default. Other supported math environments are: {{align}}, {{alignat}}, {{aligned}}, {{alignedat}}, {{array}}, {{Bmatrix}}, {{bmatrix}}, {{cases}}, {{eqnarray}}, {{equation}}, {{gather}}, {{gathered}}, {{matrix}}, {{multline}}, {{pmatrix}}, {{smallmatrix}}, {{split}}, {{subarray}}, {{Vmatrix}}, {{vmatrix}}.
L0040 
L0041 ++ [[# inline]] Inline math
L0042 
L0043 To use math expressions inside text (sentence) use {{@@[[$ ... $]]@@}} block tags.
L0044 
L0045 [[div style="float:left; width: 45%; margin: 0 2%;"]]
L0046 [[code]]
L0047 [[$ E = mc^2 $]] is much more popular than
L0048 [[$ G_{\mu\nu} - \Lambda g_{\mu\nu} = \kappa T_{\mu\nu} $]]
L0049 [[/code]]
L0050 [[/div]]
L0051 [[div style="float:left; width: 45%; margin: 0 2%;"]]
L0052 [[$ E = mc^2 $]] is much more popular than [[$ G_{\mu\nu} - \Lambda g_{\mu\nu} = \kappa T_{\mu\nu} $]]
L0053 [[/div]]
L0054 
L0055 ~~~~~~~~
```
