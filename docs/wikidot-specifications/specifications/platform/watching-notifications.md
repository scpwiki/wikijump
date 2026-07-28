# Watching and email notifications

- Feature ID: `watching-notifications`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Allow users to watch and unwatch sites, categories, pages, and forum topics, with the documented inheritance and email notification behavior.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:watching/source.wikidot.txt:1` through line 46 (canonical)

## Documentation-derived behavioral evidence

### faq:watching (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:watching/source.wikidot.txt:1` through line 46  
SHA-256 of complete source file: `6127534809cd584b7783bbbfdf87b6af452216cfee11175b064d024fb18707df`

```wikidot
L0001 +++ What is the new watching feature?
L0002 
L0003 The "watching" feature allows you to follow changes on a site, page or category. One of the methods of receiving the notifications is by email, directly into your inbox. Other delivery channels, e.g. RSS, is on its way too.
L0004 
L0005 +++ Why?
L0006 
L0007 Delivering event notifications by email has tremendous positive impact on both large communities and small teams. It helps keeping people aware of what is going on, makes the collaboration more efficient (because you do not need to exchange emails saying "Look, I have edited this page") and keeps communities more organized.
L0008 
L0009 +++ What happened to the old "watched pages" options?
L0010 
L0011 It will be removed in the near future, since it was a half-baked solution and somehow counter-intuitive.
L0012 
L0013 +++ What can be watched?
L0014 
L0015 You can watch whole sites, whole categories within a site or individual pages. If you are watching the whole site, you will be notified about changes from all pages. The same goes for category and its pages.
L0016 
L0017 If you are watching a page, we will notify you whenever someone alters a page or adds a comment to it. 
L0018 
L0019 +++ How do I start watching?
L0020 
L0021 There are 2 ways of starting watching:
L0022 
L0023 * Automatically, once you:
L0024  * create a new site, you are automatically watching the site
L0025  * edit or comment on a page, you can start watching it
L0026 * Manually
L0027  * Using the "start watching" in the bottom page options section.
L0028 
L0029 Automatic watching can be enabled/disabled in the [[[https://www.wikidot.com/account/activity#/watching| Activity / Settings]]].
L0030 
L0031 +++ Why am I suddenly watching dozens of sites and pages?
L0032 
L0033 To give this new feature proper launch all our users have been automatically subscribed to sites and pages based on the following rules:
L0034 * --if you are a member of a site, you watch it (including all the page and forum changes)--
L0035 * if you are an admin (or creator) of a site, you watch it (including all the page and forum changes)
L0036 * if you have edited or commented a page on a site you are not member of, you watch it.
L0037 
L0038 +++ How do I keep those emails organized in my mailbox?
L0039 
L0040 If you are getting more than a few notifications per day you might want to put all of them into a separate folder. If your email service (or email software) supports filters, you can easily create one. All event emails are sent from **@@watching@wikidot.com@@** address, so you can use it as the only and sufficient criteria.
L0041 
L0042 +++ How do I unwatch?
L0043 
L0044 Every notification email has a unique link. Once followed it allows to unsubscribe from a given kind (source) of events or stop all email notifications.
L0045 
L0046 You can also configure your list of watched items in your [[[https://www.wikidot.com/account/activity#/watching| Activity / Settings]]].
```
