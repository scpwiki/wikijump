# Supported browsers

- Feature ID: `browser-support`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Apply the documented browser-support policy to browser-visible Wikidot behavior.

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

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:technical/source.wikidot.txt:1` through line 24 (canonical)

## Documentation-derived behavioral evidence

### faq:technical (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:technical/source.wikidot.txt:1` through line 24  
SHA-256 of complete source file: `1a6cf585c7d13b443d9160fcde05e454b2bd50ab797b5bfd25d1f815ee3e45b1`

```wikidot
L0001 +++ Which web browsers are supported?
L0002 
L0003 At Wikidot we use graded browser support:
L0004 
L0005 ||~ Browser release date ||~ Support level ||
L0006 || last 3 years || full support ||
L0007 || 3-5 years old || partial support ||
L0008 || older than 5 years || no support ||
L0009 -----
L0010 ||~ Browser brand ||~ Support level ||
L0011 || Mozilla Firefox || full support ||
L0012 || Google Chrome || full support ||
L0013 || Apple Safari || full support ||
L0014 || mobile browsers || partial support ||
L0015 || other || partial support ||
L0016 
L0017 -----
L0018 
L0019 
L0020 full support -- everything should work
L0021 partial support -- core functions work, however glitches and errors in the presentation layer and interface might occur
L0022 no support -- Wikidot might work, but it comes without any guarantee
L0023 
L0024 To get best Wikidot experience, please use the newest browser versions available.
```
