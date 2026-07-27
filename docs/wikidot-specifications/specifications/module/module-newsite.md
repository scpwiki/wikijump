# New Site Module

- Feature ID: `module-newsite`
- Category: `module`
- Documentation status: `invocation-only`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Recognize and implement the `NewSite` module at the documented invocation sites. The corpus does not provide a dedicated module reference page.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1457/source.wikidot.txt:2` through line 2 (invocation-only)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/new-site/source.wikidot.txt:1` through line 1 (invocation-only)

## Documentation-derived behavioral evidence

### community-sites:1457 (invocation-only)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1457/source.wikidot.txt:2` through line 2  
SHA-256 of complete source file: `72891f9daaf8afc7d6a1000773c9c029272bd09d29e3ff78996d5b5da005f57e`

```wikidot
L0002 desc: '[[module NewSite]]'
```

### new-site (invocation-only)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/new-site/source.wikidot.txt:1` through line 1  
SHA-256 of complete source file: `c459571362256b5146ffaa00cfb56246428bd7eb697ebc2551cd537724b09fe6`

```wikidot
L0001 [[module NewSite]]
```
