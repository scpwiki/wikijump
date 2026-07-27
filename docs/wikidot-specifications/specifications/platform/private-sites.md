# Private sites

- Feature ID: `private-sites`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Enforce private-site visibility, membership access, unauthorized landing behavior, navigation exposure rules, and authenticated feed access.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:private-sites/source.wikidot.txt:1` through line 49 (canonical)

## Documentation-derived behavioral evidence

### faq:private-sites (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:private-sites/source.wikidot.txt:1` through line 49  
SHA-256 of complete source file: `48e9bfc04ee4d718611ae0947fa26b406ae5e3799a9e0a28effa9d41a3189954`

```wikidot
L0001 These questions are related to private sites.
L0002 
L0003 +++ What does it meen that the site is private (non-public)?
L0004 
L0005 It means that only selected Wikidot.com users (of your choice) are able to see it and browse it. You can choose the site to be private either when creating the site or later in your //Site Manager// >> //Public or Private// settings.
L0006 
L0007 +++ So who can view a private site?
L0008 
L0009 There are 2 classes of users that have "view permission":
L0010 * site members (invite them or allow them to apply for membership)
L0011 * "extra access" users you can define in //Site Manager// >> //Public or Private//
L0012 
L0013 "extra access" users will be able to view all the content from your site but will act as "Wikidot.com users" when it comes to site modifying permissions (//SiteManager// >> //Permissions//).
L0014 
L0015 In other words -- people that are not your site members and you did not give them "extra access" will not be able to browse your site but instead will see the default landing page.
L0016 
L0017 +++ What can I use the private site for?
L0018 
L0019 For whatever you want provided you comply with our Terms. People use it for personal notepads or workspaces, teachers use it for their classes, some people use it for online collaboration...
L0020 
L0021 I myself ([[user michal frackowiak]]) am using a private site to take notes online, store ideas, keep my "todo" lists, prepare things for my students...
L0022 
L0023 Just think what you need a private space for ;-)
L0024 
L0025 +++ How many users (members) are allowed for a private Wiki?
L0026 
L0027 At the moment free private Wikis have a limit of **5 members** + 5 extra access permissions. To increase the limit, please upgrade your Account. For more information please refer to the [http://www.wikidot.com/plans Plans] or [http://www.wikidot.com/faq:upgrades Upgrade's FAQ].
L0028 
L0029 +++ Is this really secure?
L0030 
L0031 Yes, the system has been carefully designed and data from your private site should not leak unless someone can steal your password and/or access Wikidot.com as someone who has sufficient permissions.
L0032 
L0033 The system is much better with the SSL encryption. It's available for Pro+ Accounts.
L0034 
L0035 +++ Although I have specified the default landing page for unauthorized users there are still top- and side-bar menus there that reveal some of my content!
L0036 
L0037 Indeed and this might be a problem when you want to keep your nav:top and nav:side secret. Here is what to do:
L0038 * create a landing page within a new category, e.g. {{unauthorized:start}}
L0039 * go to the Site Manager >> Appearance >> Navigation element
L0040 * clear navigational elements for category //unauthorized//.
L0041 * go to Site Manager >> Public of private and set {{unauthorized:start}} as a landing page.
L0042 
L0043 Update: now there is an option to disable the nav elements in your Site Manager >> Public or Private. However the method described above is visually better (often).
L0044 
L0045 +++ How do I access RSS feeds from private sites?
L0046 
L0047 All the RSS feeds from private sites are password-protected and only members of the given site are allowed to access them. The authentication mechanism is HTTP Basic Authentication -- supported by most of the feed readers.
L0048 
L0049 However due to security reasons the password for accessing the feed is **not the same as your log-in password**. As a user name you should still put your email address but for the password please use the same password you are using for your private feed: please go to [*http://www.wikidot.com/account:you/start/settings Settings in My Account] and click on Notifications to find out the password.
```
