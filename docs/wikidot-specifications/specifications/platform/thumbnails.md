# Page and site thumbnails

- Feature ID: `thumbnails`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Generate and serve the documented thumbnail URL forms and size variants.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:thumbnails/source.wikidot.txt:1` through line 28 (canonical)

## Documentation-derived behavioral evidence

### doc:thumbnails (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:thumbnails/source.wikidot.txt:1` through line 28  
SHA-256 of complete source file: `4673c5df579648bef01a104fd68da19852e02d722ecb585a2753385c2471582c`

```wikidot
L0001 Wikidot automatically generates thumbnails in various sizes for:
L0002 
L0003 * every Wikidot site
L0004 * every theme on [http://themes.wikidot.com the themes project]
L0005 
L0006 The URL for site thumbnail is:
L0007 
L0008 @@http://thumbnail.wdfiles.com/thumbnail/site/@@//<site_domain_name>//@@/@@//<size>//@@.jpg@@
L0009 
L0010 <site_domain_name>: the main URL for site, for example: www.wikidot.com
L0011 <size>: one of 160, 80, 40, 20
L0012 
L0013 The URL for theme thumbnail is:
L0014 
L0015 @@http://thumbnail.wdfiles.com/thumbnail/theme/@@//<theme_name>//@@/@@//<size>//@@.jpg@@
L0016 
L0017 <theme_name>: the unix name of theme
L0018 <size>: one of 500, 240, 160, 80
L0019 
L0020 Examples:
L0021 
L0022 * http://thumbnail.wdfiles.com/thumbnail/site/sfugamedev.wikidot.com/160.jpg
L0023 
L0024 [[=image https://thumbnail.wdfiles.com/thumbnail/site/sfugamedev.wikidot.com/160.jpg]]
L0025 
L0026 * http://thumbnail.wdfiles.com/thumbnail/theme/curvature/240.jpg
L0027 
L0028 [[=image https://thumbnail.wdfiles.com/thumbnail/theme/curvature/240.jpg]]
```
