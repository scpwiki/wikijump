# Educational site status

- Feature ID: `educational-site-status`
- Category: `platform`
- Documentation status: `documented-plan-capability`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Support the documented educational-site eligibility, application authority, storage, file-size, membership, revision, HTTPS, analytics, cost, and upgrade interaction rules.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/education/source.wikidot.txt:43` through line 65 (canonical)

## Documentation-derived behavioral evidence

### education (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/education/source.wikidot.txt:43` through line 65  
SHA-256 of complete source file: `637d504390cbc201bb079163ba6b9135a23f06efc781b03b5ebcd67b55c4ffe1`

```wikidot
L0043 [[div class="well"]]
L0044 The **Educational status** of your site can give you extra:
L0045 * unlimited number of members even if your site is //private//
L0046 * 25 GB for file uploads, each file up to 100 MB, more storage per request
L0047 * SSL security 
L0048 * unlimited number of revisions per page
L0049 * web traffic analytics
L0050 * other small improvements
L0051 
L0052 The Educational sites are absolutely **free** for educational / research purposes. Please create your site, enter Site Manager and look for Educational upgrade. 
L0053 
L0054 Please note that you need to be Master Administrator of the Site to apply for the educational status.
L0055 
L0056 If you upgrade your account, Edu sites will retain its status and won't be upgraded to Pro. In order to upgrade them as well, please send an email at support@wikidot.com
L0057 
L0058 [[/div]]
L0059 
L0060 
L0061 [[div style="font-size: 85%"]]
L0062 Who is eligible:
L0063 * educators, researchers, school and academic teachers using Wikidot for research purposes or communicating with students
L0064 * students and pupils for any activity associated with school or academic projects
L0065 * the educational upgrade of individual sites cannot be combined with [[[plans | premium account upgrades]]]
```
