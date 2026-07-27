# Wikidot users and site roles

- Feature ID: `user-roles`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Distinguish anonymous users, registered users, site members, moderators, administrators, and superusers with the documented status relationships.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:users/source.wikidot.txt:1` through line 42 (canonical)

## Documentation-derived behavioral evidence

### doc:users (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:users/source.wikidot.txt:1` through line 42  
SHA-256 of complete source file: `8458b5548c382222fdf224e38b49de070b024f2ce35c6d031b687455582f5686`

```wikidot
L0001 This document describes how User accounts, roles and permissions are organized within the Wikidot.com network.
L0002 
L0003 [[toc]]
L0004 
L0005 + One account - access many sites 
L0006 
L0007 At the WikiDot.com network all the sites share the same User accounts. It means that having one account guarantees the same User identity at all the sites. 
L0008 
L0009 + User status within a site
L0010 
L0011 ++ Anonymous User
L0012 
L0013 Some sites may allow anonymous users (i.e. these who do not use a valid WikiDot account - are not logged in) to modify content and use discussion forum. In any such case the visitor's IP address will be stored and publicly visible.
L0014 
L0015 ++ Registered Users
L0016 
L0017 Registered Users are these who have and use a valid Wikidot account but are not necessarily a member of a given site. Some sites allow such Users to modify content and use forum.
L0018 
L0019 ++ Members of the site
L0020 
L0021 Members of the particular site are these who through some process joined the site. One can become a Member of a site by
L0022 * applying to Site Administrators (if enabled)
L0023 * by providing a valid //membership password// (if enabled)
L0024 * by accepting an invitation from a Site Administrator
L0025 
L0026 Some sites allow content modification and forum postings only to its Members. In fact this is the default ;-)
L0027 
L0028 +++ Site Administrators
L0029 
L0030 Users who have all possible permissions within a site. A User who creates a new site also becomes an Administrator of this site. Other site Members can be also given Administrator roles.
L0031 
L0032 +++ Site Moderators
L0033 
L0034 Users who have certain permissions to modify content but can not access site settings. Moderators can have any of the two roles
L0035 * Page Moderator - can modify content pages
L0036 * Forum Moderators - can modify (edit, delete) forum threads and posts.
L0037 
L0038 Site Moderators are given their roles by Site Administrators.
L0039 
L0040 + WikiDot.com Superusers
L0041 
L0042 There is also a group of Superusers which most possibly belong to Wikidot.com staff. They can do a lot.
```
