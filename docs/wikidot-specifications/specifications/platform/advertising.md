# Site advertising

- Feature ID: `advertising`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Apply Wikidot's documented advertising placement and account/site eligibility behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:advertising/source.wikidot.txt:1` through line 13 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:43` through line 47 (supporting)

## Documentation-derived behavioral evidence

### doc:advertising (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:advertising/source.wikidot.txt:1` through line 13  
SHA-256 of complete source file: `d75b36abb25683426ef5e9a734bd8aa7573f1722ac1c818d56ceb6268a8e0f39`

```wikidot
L0001 At Wikidot.com we display 3rd party advertising on free sites to help us pay the bills and keep the most services free.
L0002 
L0003 Our algorithm tries to show ads in a non-disturbing way, e.g. logged-in users should not see any ads at all. We try to optimize ads so that they do not impair the user experience, but they need to be visible enough so that placing advertising on Wikidot sites provides a value for advertisers.
L0004 
L0005 The algorithm will put more ads on sites that show no recent activity -- no page edits, comments nor forum posts.
L0006 
L0007 New sites might not show ads until they have enough content and traffic.
L0008 
L0009 To remove ads from your site, there are two simple choices:
L0010 # Upgrade to one of our [[[plans | Pro plans]]] -- even Pro Lite (for about $5 a month) removes ads from sites you are a master administrator of. Moreover, you can control advertising yourself and earn money from your traffic.
L0011 # If you are running an educational site (for your classes, student projects etc.) you could apply for the [[[education | Educational upgrade]]].
L0012 
L0013 Also, ads are not displayed on any private sites.
```

### features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/features/source.wikidot.txt:43` through line 47  
SHA-256 of complete source file: `2f543ffe5d97f77da4936b7ab95ac66493b1acedd2bea01d5b956735b1b9501c`

```wikidot
L0043 +++ CONTROL OVER ADS
L0044 Although from time to time we [[[doc:advertising | display ads on free sites]]], this is done as discretely as possible. Moreover, we are giving you simple opt-out options which transfer all the control to you! If you want, you can start earning money with Google AdSense whenever someone clicks ads on your pages. The best thing is that you can set if the ads are displayed to everyone or only to unlogged Wikidot users. You also have control over placement, colors, sizes -- and you can start [[[adsense | now!]]]
L0045 
L0046 
L0047 
```
