# Secure login

- Feature ID: `secure-login`
- Category: `platform`
- Documentation status: `high-level-documentation`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot capability “Secure login” and its user-visible configuration, state, permissions, and output.

## Implementation contract

- The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.
- Account, site, category, page, and actor context MUST be enforced at the public service boundary.
- Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- The corpus describes this capability at product level. Use live Wikidot evidence to resolve any implementation detail the snapshot does not define.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:126` through line 130 (supporting)

## Documentation-derived behavioral evidence

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:126` through line 130  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0126 +++ SECURE SSL LOGIN
L0127 When using Wikidot.com you can be sure your private data is transmitted safely! No more plain-text passwords.
L0128 
L0129 
L0130 
```
