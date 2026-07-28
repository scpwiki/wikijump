# Site navigation

- Feature ID: `site-navigation`
- Category: `platform`
- Documentation status: `high-level-documentation`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented Wikidot capability “Site navigation” and its user-visible configuration, state, permissions, and output.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:91` through line 95 (supporting)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/nav:side/source.wikidot.txt:1` through line 3 (site-navigation-example)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/nav:top/source.wikidot.txt:1` through line 6 (site-navigation-example)

## Documentation-derived behavioral evidence

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:91` through line 95  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0091 +++ EASY NAVIGATION AND USER INTERFACE
L0092 There is a simple way to create your own menus: top and side. Creating navigation elements and links is very simple. Adding breadcrumbs and sitemap to your site will make navigation even more clear and comprehensive.
L0093 
L0094 
L0095 
```

### nav:side (site-navigation-example)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/nav:side/source.wikidot.txt:1` through line 3  
SHA-256 of complete source file: `bef657de2ecbd77670964744557f49d43a70406a9f015b26390a5556caa5ac5f`

```wikidot
L0001 * [[[_admin|Site Manager]]]
L0002 
L0003 = [[size 80%]][/nav:side/edit/true Edit this menu][[/size]]
```

### nav:top (site-navigation-example)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/nav:top/source.wikidot.txt:1` through line 6  
SHA-256 of complete source file: `4017a46ec43cdc9c16ea2f33a79c0a1feb60c37be26c7aa6751c6f0350001516`

```wikidot
L0001 [[ul class="nav navbar-nav navbar-right"]]
L0002       [[li]][[a href="/more:explore-features"]]Features[[/a]][[/li]]
L0003       [[li]][[a href="/more:testimonials"]]Opinions[[/a]][[/li]]
L0004       [[li]][[a href="/plans"]]Pricing[[/a]][[/li]]
L0005       [[li]][[a href="/advertise"]]Advertising[[/a]][[/li]]
L0006 [[/ul]]
```
