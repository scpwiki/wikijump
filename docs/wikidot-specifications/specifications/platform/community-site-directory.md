# Community Site directory and application

- Feature ID: `community-site-directory`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Represent Community Sites, their application and ownership rules, advertising rules, deletion constraints, and directory records stored as structured page data.

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
- Data-form template parsing and saved page rendering
- Public create/edit/view flow and ListPages query behavior where documented

## Feature-specific implementation notes

- The corpus contains 1,560 `community-sites:*` records. The representative ranges document non-free-text record fields; source-coverage.json inventories every record without copying user-submitted descriptions or contact details into the specification.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites/source.wikidot.txt:1` through line 24 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1/source.wikidot.txt:1` through line 1 (representative-data-record)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1/source.wikidot.txt:3` through line 6 (representative-data-record)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:community-sites/source.wikidot.txt:1` through line 65 (canonical)

## Documentation-derived behavioral evidence

### community-sites (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites/source.wikidot.txt:1` through line 24  
SHA-256 of complete source file: `63c3b5e0ca4982adb42003fb2b274bf1c663c3b0b916550284ed867c4abf5f35`

```wikidot
L0001 [[f>image 20121019-8xytm9jam449sngnkfbausq3r9.jpeg width="450px" style="margin: 35px 60px; box-shadow: 3px 3px 15px black;"]]
L0002 
L0003 [[div style="text-align: justify; width: 480px; padding: 30px;"]]
L0004 = [[size 18px]]**Community Sites** consist of[[/size]]
L0005 = [[size 26px]]**most popular, high-quality, community-driven  projects at Wikidot**[[/size]]
L0006 
L0007 [[size 18px]]
L0008  They combine the best of free and paid plans:
L0009 
L0010 * Free to run
L0011 * More capable than Pro Plus 
L0012 * Migration support from Wikidot Team
L0013 [[/size]]
L0014 
L0015 [[size 18px]]
L0016 Have a community-driven wiki or want to migrate one to our platform
L0017 [[/size]]
L0018 
L0019 = [[size 30px]]**[[[/community-sites:new/edit/true | Apply here]]]**[[/size]]
L0020 
L0021 [[size 14px]]
L0022 For more information, please visit [[[faq:community-sites | Community Sites FAQ and documentation]]].
L0023 [[/size]]
L0024 [[/div]]
```

### community-sites:1 (representative-data-record)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1/source.wikidot.txt:1` through line 1  
SHA-256 of complete source file: `b8c3cdb12fed29bd642c82ac1361aac53d660138e903b09ea1ae805da0075afc`

```wikidot
L0001 address: downwave.wikidot.com
```

### community-sites:1 (representative-data-record)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/community-sites:1/source.wikidot.txt:3` through line 6  
SHA-256 of complete source file: `b8c3cdb12fed29bd642c82ac1361aac53d660138e903b09ea1ae805da0075afc`

```wikidot
L0003 state: idea
L0004 feature_support: '1'
L0005 feature_promotion: '0'
L0006 feature_features: '0'
```

### faq:community-sites (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:community-sites/source.wikidot.txt:1` through line 65  
SHA-256 of complete source file: `66dcac7ef057faaa1fc886f6824f6217d15ec0d61d4292edc2c712f8546f979a`

```wikidot
L0001 +++ What are Community Sites?
L0002 
L0003 Community Sites consist of most popular, high-quality and community-driven projects at Wikidot. Community Sites combine the best of our free and paid plans:
L0004 
L0005 * Free to run
L0006 * More capable than Pro Plus 
L0007 * Migration support from Wikidot Team
L0008 
L0009 Very often these above-the-line sites gather growing communities and require extra resources. By becoming a Community Site each of them is guaranteed to get resources required not only to bypass limits of free or paid plans, but also to become (with our help if requested) a top-quality wiki. 
L0010 
L0011 Moreover, the Community Sites concept addresses the problem of free (or even paid) sites quickly becoming so large that it requires a higher paid plan. Which is not always acceptable for fan-driven communities.
L0012 
L0013 
L0014 +++ Why is Wikidot investing in Community Sites?
L0015 
L0016 We constantly see admins of free (or even paid) sites asking us for extra features required by their growing sites. We have been helping them in numerous cases because we believe that every successful site is our success too. We have already seen that this works!
L0017 
L0018 We see the Community Sites category as an accelerator that would bootstrap new great sites and help existing sites become even better.
L0019 
L0020 We are open to specific needs of particular sites and (if required) we will implement features and improvements to the Wikidot platform.
L0021 
L0022 +++ What are the conditions to get a Community Site status and how to do it?
L0023 
L0024 Obtaining a Community Site status is a simple process:
L0025 
L0026 1. Fill an [[[community-sites | online application]]]
L0027 2. We will get back to you with more details
L0028 
L0029 The applications are processed by the Wikidot Team. To be accepted a site should:
L0030 
L0031 * contain quality, original content
L0032 * be driven by an active community
L0033 * be publicly available with either open or closed membership
L0034 
L0035 +++ Who owns a Community Site and why is there no Master Admin?
L0036 
L0037 As opposed to a regular Wikidot site, a Community Site does not have an owner (aka Master Admin). It is run by Admins, Moderators and Members, but does not depend on a single, particular person. This, judging from our past experience with other sites, vastly improves the continuity of the site by distributing the responsibilities to the group of admins.
L0038 
L0039 +++ How many admins a Community Site should have?
L0040 
L0041 We encourage more than one admin to guarantee continuity of the site.
L0042 
L0043 +++ I have a paid account but want to convert my sites to Community Sites
L0044 
L0045 No problem. Paid sites can be converted to Community Sites too. Moreover, if you decide that you do not need your paid account any more, we can issue a refund proportional to the remaining time of your paid plan.
L0046 
L0047 +++ Does my Community Site affect my free/paid account?
L0048 
L0049 No. Community Sites are separated from your personal plan and do not affect your account. Neither storage nor traffic of any Community Site is included in your account usage.
L0050 
L0051 +++ Can a Community Site be deleted?
L0052 
L0053 No. Certain options, including renaming and removing, are disabled for Community Sites.
L0054 
L0055 +++ Can I take over an abandoned Community Site?
L0056 
L0057 Yes! Each such case is processed individually, so just get in touch with us!
L0058 
L0059 +++ Running advertising on Community Sites
L0060 
L0061 As with free sites and high-traffic paid sites, we reserve the right to run advertising on Community Sites. However, ads will never be shown to any logged-in users, so your community should not be affected at all.
L0062 
L0063 +++ I have a great idea for a Community Site!
L0064 
L0065 Let us know by filling the [[[community-sites | application]]]. If you can convince us to your idea we might help you build your Community Site!
```
