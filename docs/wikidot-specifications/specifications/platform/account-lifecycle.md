# User account lifecycle and authentication recovery

- Feature ID: `account-lifecycle`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Support account eligibility, deletion, and documented recovery from authentication state problems.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:user-accounts/source.wikidot.txt:1` through line 20 (canonical)

## Documentation-derived behavioral evidence

### faq:user-accounts (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:user-accounts/source.wikidot.txt:1` through line 20  
SHA-256 of complete source file: `1719c7220c310476a458d07df8048ed92c83d2c61994fb11758327a4e9939c66`

```wikidot
L0001 +++ Who can create a User account?
L0002 
L0003 From our point of view - anyone. From yours - please consult your local law, boss, parents or whatever.
L0004 
L0005 +++ Can I delete my user account?
L0006 
L0007 Yes, you can delete your Account
L0008 
L0009 * go to [*https://www.wikidot.com/account/settings Your Account Settings] and go to the **Account Settings**
L0010 * click **Delete Account** at the bottom of the list and follow the instructions. You will receive a confirmation e-mail and after clicking on the link in the e-mail, you will be prompted to provide your password. After doing so, your account will be deleted.
L0011 
L0012 **##red|Note that deleting an account is not equal to deleting Wikis.##** If you will delete an account, you Wiki will still be hosted by Wikidot.com.
L0013 
L0014 +++ Help! I experience strange login/logout/authentication behavior!
L0015 
L0016 Some of the problems related to session handling might be caused (and very often are) by strange configuration provided to you by your internet providers -- such as routing problems, forcing caching, short dynamic IP lease time, filtering cookies etc.
L0017 
L0018 If you experience any problems with your browser "logging you out randomly" -- please try to experiment with the log-in options. In most cases this would help:
L0019 * bind session to my IP -- set to //no//
L0020 * do not timeout my session -- set to //yes//
```
