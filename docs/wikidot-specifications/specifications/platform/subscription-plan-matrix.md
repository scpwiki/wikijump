# Subscription plan comparison

- Feature ID: `subscription-plan-matrix`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Display the documented plan capabilities, prices, limits, and comparison matrix.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:1` through line 206 (canonical)

## Documentation-derived behavioral evidence

### plans (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/plans/source.wikidot.txt:1` through line 206  
SHA-256 of complete source file: `ae76ced159baf8ac10913fcef88ef4beab8b8b3bc600f55556ee90e4b4991b9c`

```wikidot
L0001 [[module CSS]]
L0002 .flat .plan {
L0003   border-radius: 6px;
L0004   list-style: none;
L0005   padding: 0 0 20px;
L0006   margin: 0 0 15px;
L0007   background: #fff;
L0008   text-align: center;
L0009 }
L0010 .flat .plan li {
L0011   padding: 10px 15px;
L0012   color: #000;
L0013   border-top: 1px solid #f5f5f5;
L0014   -webkit-transition: 300ms;
L0015   transition: 300ms;
L0016 }
L0017 .flat .plan li.plan-price {
L0018   border-top: 0;
L0019   font-size: 32px;
L0020   font-weight: 100;
L0021 }
L0022 .flat .plan li.plan-name {
L0023   border-radius: 6px 6px 0 0;
L0024   padding: 15px;
L0025   font-size: 24px;
L0026   line-height: 24px;
L0027   color: #fff;
L0028   background: #e74c3c;
L0029   margin-bottom: 10px;
L0030   border-top: 0;
L0031   font-weight: 200;
L0032 }
L0033 .flat .plan li > strong {
L0034   color: #e74c3c;
L0035 }
L0036 
L0037 .flat .plan li.plan-name strong {
L0038   color: #000;
L0039 }
L0040 .flat .plan li.plan-action {
L0041   margin-top: 10px;
L0042   border-top: 0;
L0043 }
L0044 .flat .plan.featured {
L0045   -webkit-transform: scale(1.1);
L0046   -ms-transform: scale(1.1);
L0047   transform: scale(1.1);
L0048 }
L0049 .flat .plan.featured li.plan-name {
L0050   background: #000;
L0051 }
L0052 .flat .plan.featured li.plan-name strong {
L0053   color: #FFF;
L0054 }
L0055 #footer {
L0056   margin-top: 100px;
L0057   padding-bottom: 30px;
L0058 }
L0059 [[/module]]  
L0060 
L0061 [[module CurrencyConvert]]
L0062  [[div class="container"]]
L0063         [[div class="row flat"]]
L0064             [[div class="col-lg-3 col-md-3"]]
L0065                 [[ul class="plan plan1"]]
L0066                     [[li class="plan-name"]]
L0067                         **Pro Lite** account
L0068                     [[/li]]
L0069                     [[li class="plan-price"]]
L0070                         $49.90
L0071                     [[/li]]
L0072                     [[li]]
L0073                         **For personal websites, blogs, community sites.**
L0074                     [[/li]]
L0075                     [[li]]
L0076                         Create up to 5 sites and use up to 30 GB of storage for your files, invite your friends and coworkers. Create private sites with up to 10 members each.
L0077                     [[/li]]
L0078                     [[li class="plan-action"]]
L0079                         [[a href="/account/upgrade" class="btn btn-danger btn-lg"]]Upgrade[[/a]]
L0080                     [[/li]]
L0081                 [[/ul]]
L0082             [[/div]]
L0083 
L0084             [[div class="col-lg-3 col-md-3"]]
L0085                 [[ul class="plan plan2 featured"]]
L0086                     [[li class="plan-name"]]
L0087                         **Pro** account
L0088                     [[/li]]
L0089                     [[li class="plan-price"]]
L0090                         $119.90
L0091                     [[/li]]
L0092                     [[li]]
L0093                         **For business websites, medium-sized private workgroups.**
L0094                     [[/li]]
L0095                     [[li]]
L0096                         Create up to 10 sites and use up to 100 GB of storage, with files up to 50 MB each. Create private sites with up to 20 members each, and get built-in web traffic analysis.
L0097                     [[/li]]
L0098                     [[li class="plan-action"]]
L0099                      [[a href="/account/upgrade" class="btn btn-danger btn-lg"]]Upgrade[[/a]]
L0100                  [[/li]]
L0101              [[/ul]]
L0102          [[/div]]
L0103 
L0104          [[div class="col-lg-3 col-md-3"]]
L0105             [[ul class="plan plan3"]]
L0106                 [[li class="plan-name"]]
L0107                     **Pro Plus** account 
L0108                 [[/li]]
L0109                 [[li class="plan-price"]]
L0110                     $239.90
L0111                 [[/li]]
L0112                 [[li]]
L0113                     **For business-critical web sites.**
L0114                 [[/li]]
L0115                 [[li]]
L0116                     Create up to 30 sites, and use up to 200 GB of storage. Create private sites with unlimited membership, get SSL secure access, and receive priority email support for all your questions.
L0117                 [[/li]]
L0118                 [[li class="plan-action"]]
L0119                  [[a href="/account/upgrade" class="btn btn-danger btn-lg"]]Upgrade[[/a]]
L0120              [[/li]]
L0121          [[/ul]]
L0122      [[/div]]
L0123 
L0124      [[div class="col-lg-3 col-md-3"]]
L0125         [[ul class="plan plan4"]]
L0126             [[li class="plan-name"]]
L0127                 **Community** Site
L0128             [[/li]]
L0129             [[li class="plan-price"]]
L0130                 FREE!
L0131             [[/li]]
L0132             [[li]]
L0133                 **For quality community-driven sites**
L0134             [[/li]]
L0135             [[li]]
L0136                 Practically unlimited resources, including storage, traffic, number of members, plus direct support from the Wikidot Team. Only available for high-quality, approved sites.
L0137             [[/li]]
L0138             [[li class="plan-action"]]
L0139              [[a href="/faq:community-sites" class="btn btn-danger btn-lg"]]Learn more[[/a]]
L0140          [[/li]]
L0141      [[/ul]]
L0142  [[/div]]
L0143 [[/div]]
L0144 
L0145 [[/div]]
L0146 [[/module]]
L0147 
L0148 Wikidot.com started in 2006 as a free Wiki provider.  Today, we have a range of plans that suit your use of Wikidot, from casual user to dedicated professional and small business.
L0149 
L0150 + Detailed comparison
L0151 
L0152 [[div class="plans-table" style="margin: 0 40px;"]]
L0153 [[module CurrencyConvert]]
L0154 ||~ ||~ Free account ||~ Pro Lite account ||~ Pro account ||~ Pro+ account ||~ Community Site||
L0155 || Price for 12 months of service ||= free ||= $49.90 ||= $119.90 ||= $239.90 ||= free ||
L0156 || Number of sites ||= 5 ||= 5 ||= 10 ||= 30 ||= - ||
L0157 || Storage ||= 5 x 300MB ||= 30 GB ||= 100 GB ||= 200 GB||= unlimited[[footnote]]Limits increased upon request[[/footnote]]||
L0158 || Account-based storage calculation ||= - ||= yes _
L0159 [[size 80%]](30 GB + 5 GB per purchased slot)[[/size]] ||= yes _
L0160 [[size 80%]](100 GB + 5 GB per purchased slot)[[/size]] ||= yes _
L0161 [[size 80%]](200 GB + 5 GB per purchased slot)[[/size]] ||= - ||
L0162 || Can buy more sites and storage (slots) ||= - ||= yes||= yes ||= yes ||= - ||
L0163 || New features earlier ||= - ||= yes ||= yes ||= yes ||= yes ||
L0164 || Hide own karma and pro status ||= - ||= - ||= - ||= yes ||= - ||
L0165 || Support ||= community ||= community ||= email ||= priority email ||= priority email ||
L0166 ||||||||||~ [[size 130%]]Features that apply to all sites created by a User[[/size]] || ||
L0167 ||~ ||~ Free account ||~ Pro Lite account ||~ Pro account ||~ Pro+ account ||~ Community Site ||
L0168 || Max number of pages ||= unlimited ||= unlimited ||= unlimited ||= unlimited ||= unlimited ||
L0169 || Max number of revisions per page ||= unlimited ||=  unlimited ||= unlimited ||= unlimited ||= unlimited ||
L0170 || Custom CSS themes ||= yes ||= yes ||= yes ||= yes ||= yes||
L0171 || Simple backups ||= yes ||= yes ||= yes ||= yes ||= yes ||
L0172 || Max members of a public site ||= unlimited ||= unlimited ||= unlimited ||= unlimited ||= unlimited ||
L0173 || Advanced membership, roles and permissions ||= yes ||= yes ||= yes ||= yes ||= yes ||
L0174 || Can make a site private (non public) ||= yes ||= yes ||= yes ||= yes ||= no ||
L0175 || Max number of members for private sites ||= 5 ||= 10 ||= 20 ||= unlimited ||= - ||
L0176 || Max upload storage per site ||= 300 MB ||= flexible ||= flexible ||= flexible ||= unlimited[[footnote]]Limits increased upon request[[/footnote]] ||
L0177 || Max uploaded file size ||= 25 MB ||= 50 MB ||= 100 MB ||= 200 MB ||= 200 MB||
L0178 || Custom domain mapping ||= yes ||= yes ||= yes ||= yes ||= yes||
L0179 || **Advertising** ||= **no control** ||= **full control** ||= **full control** ||= **full control** ||= limited control||
L0180 || Custom layout ||= - ||= yes ||= yes ||= yes ||= yes ||
L0181 || XML-RPC api ||= - ||= yes ||= yes ||= yes ||= yes ||
L0182 || Block cloning of site ||= - ||= yes ||= yes ||= yes ||= yes ||
L0183 || Block inclusion of site ||= - ||= yes ||= yes ||= yes ||= yes ||
L0184 || Advanced web statistics ||= - ||= yes ||= yes ||= yes ||= yes ||
L0185 || Per-site user profiles ||= - ||= - ||= yes ||= yes ||= yes||
L0186 || Hide karma and pro status icons for all users within a site ||= - ||= - ||= - ||= yes ||= yes||
L0187 || Custom footer ||= - ||= - ||= - ||= yes ||= yes ||
L0188 || Secure access via https (SSL) ||= - ||= - ||= - ||= yes ||= upon request||
L0189 
L0190 Pro Lite, Pro and Pro+ users can purchase additional slots (1 extra site + 5 GB of storage) in packages: 5 slots for $59.90 in a yearly subscription.
L0191 [[/module]]
L0192 
L0193 Sites generating extremely high traffic may be priced individually.
L0194 
L0195 [[size 85%]]Prices in Euro (€) are valid for European Union Citizens and contain 23% VAT. Prices in United States Dollar (US$) are valid for customers outside of EU and do not contain VAT.[[/size]]
L0196 
L0197 [[/div]]
L0198 
L0199 
L0200 = [[a class="btn btn-lg btn-warning" href="https://www.wikidot.com/account/upgrade"]]Upgrade your account now![[/a]]
L0201 
L0202 
L0203 * For each purchase Wikidot provides full VAT invoices.
L0204 * All upgrades are eligible for 30-day "no questions asked" refund.
L0205 
L0206 See [http://www.wikidot.com/faq:upgrades Upgrades and Plans FAQ] and [http://www.wikidot.com/faq:community-sites Community Sites FAQ] for more details.
```
