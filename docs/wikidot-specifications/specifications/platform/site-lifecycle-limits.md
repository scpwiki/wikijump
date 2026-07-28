# Site limits, backup, anti-abuse, deletion, and restoration

- Feature ID: `site-lifecycle-limits`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented site ownership limits, storage/page limits, backup behavior, vandalism controls, founder-only deletion, and deletion undo.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:site-features/source.wikidot.txt:1` through line 39 (canonical)

## Documentation-derived behavioral evidence

### faq:site-features (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:site-features/source.wikidot.txt:1` through line 39  
SHA-256 of complete source file: `ba839b11f0ee2c44ac47afc53264a78701ede052312309cfa8687f8368be4e1c`

```wikidot
L0001 +++ I already have a site and users. Can a Wikidot Site use external user accounts?
L0002 
L0003 No, not at this moment. We have however some ideas how to implement it through distributed and modular authentication network but do not expect anything soon.
L0004 
L0005 If you want to integrate your new Wikidot Site with your existing project you can easily create a subdomain within your project's domain and let Wikidot.com handle it.
L0006 
L0007 +++ Do you force me to display any ads?
L0008 
L0009 Yes, we show advertising to selected visitors of your sites. This helps maintaining free sites and keeping them free for you.
L0010 
L0011 +++ Is there any limit on the __size__ of a Site?
L0012 
L0013 There is only a limit for the file storage, i.e. total space for the file attachments. By default it is 100MB - which is low but should be higher soon.
L0014 
L0015 There is absolutely no limit on number of pages you want to create, number of Site Members or any other factors. We would gladly host your verrrrrrry big Site ;-)
L0016 
L0017 +++ How many Sites can I create?
L0018 
L0019 At the moment any User can create (or administer) up to 5 sites. This limit is much higher for our paying users.
L0020 
L0021 +++ Can I have a backup of my Site please?
L0022 
L0023 Yes, you can make a snapshot of your Site that would contain sources for all the pages you have created. Although this has some limitations it should be a nice thing for these who would like to have their content a bit safer ;-)
L0024 
L0025 Although we do not guarantee that your content is safe on our servers (i.e. take no responsibility for any data loss) we follow a strict backup policy which include redundant disk storage, hot-backup server (synced with the master servers in real-time) and regular backups to servers in a different data center.
L0026 
L0027 +++ How can I fight spam and vandalism?
L0028 
L0029 There are a few ways:
L0030 
L0031 * limiting permissions - allow only trusted users to modify content
L0032 * setting less restrictive permissions and monitoring changes (also via an RSS feed) and reverting changes
L0033 * blocking vandals from accessing the site
L0034 
L0035 +++ How can I delete a Site?
L0036 
L0037 Yes, you can. There is an option to delete a wiki inside the Site Manager under "Extreme actions". Moreover for every site deletion there is an **undo** -- you can easily restore previously deleted wiki. 
L0038 
L0039 Only a person who originally created the wiki (the founder) can delete it.
```
