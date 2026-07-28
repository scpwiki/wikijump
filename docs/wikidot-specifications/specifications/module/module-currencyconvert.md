# Currency Convert Module

- Feature ID: `module-currencyconvert`
- Category: `module`
- Documentation status: `invocation-only`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Recognize and implement the `CurrencyConvert` module at the documented invocation sites. The corpus does not provide a dedicated module reference page.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- The documentation corpus proves the module name and invocation context, but not a complete behavior contract.
- Before implementing behavior beyond the recorded invocation, capture live Wikidot output at the public rendering or browser seam and add that evidence to this specification.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:61` through line 61 (invocation-only)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:153` through line 153 (invocation-only)

## Documentation-derived behavioral evidence

### plans (invocation-only)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:61` through line 61  
SHA-256 of complete source file: `ae76ced159baf8ac10913fcef88ef4beab8b8b3bc600f55556ee90e4b4991b9c`

```wikidot
L0061 [[module CurrencyConvert]]
```

### plans (invocation-only)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:153` through line 153  
SHA-256 of complete source file: `ae76ced159baf8ac10913fcef88ef4beab8b8b3bc600f55556ee90e4b4991b9c`

```wikidot
L0153 [[module CurrencyConvert]]
```
