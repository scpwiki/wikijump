# Collaborative page and file editing

- Feature ID: `collaborative-editing`
- Category: `platform`
- Documentation status: `high-level-documentation`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Allow authorized users to create and edit shared pages, publish changes, collaborate on documents, and share files through a site.

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

- The corpus states the collaborative capability but does not define concurrent-edit conflict semantics. Capture live behavior before choosing a conflict model.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/education/source.wikidot.txt:20` through line 32 (supporting)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/inc:awesome-features/source.wikidot.txt:22` through line 30 (supporting)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/inc:what-is-wikidot/source.wikidot.txt:5` through line 6 (supporting)

## Documentation-derived behavioral evidence

### education (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/education/source.wikidot.txt:20` through line 32  
SHA-256 of complete source file: `637d504390cbc201bb079163ba6b9135a23f06efc781b03b5ebcd67b55c4ffe1`

```wikidot
L0020 You can:
L0021 * create pages with news, resources, marks etc.
L0022 * create and edit pages with others,
L0023 * start common-created projects and collaboration,
L0024 * search the history of changes, you can compare revisions of any page and you will never loose your content!
L0025 * easily upload files, images, documents, notes and [[size large]]**share it**[[/size]] with your students and classmates,
L0026 * use [[size large]][*http://www.wikidot.com/doc-wiki-syntax:math mathematical equations], [*http://www.wikidot.com/doc-wiki-syntax:bibliography bibliography items], [*http://www.wikidot.com/doc-wiki-syntax:footnotes footnotes][[/size]] and more...
L0027 * easily organize your Site with categories, lists, menus,
L0028 * use **[http://www.wikidot.com/doc:modules Modules]** -- our killer feature -- that easily add interactivity to wiki pages,
L0029 * make your Site private and accessible only for you and your students,
L0030 * change the theme of the Site, you can shoose from many modern, nice-looking themes,
L0031 
L0032 You can also enrich your pages with videos, music, podcasts, imported RSS feeds, Flickr.com images, del.icio.us bookmarks, content from YouTube and other video sharing portals. Extend your pages using a growing number of Modules that will help you build truly interactive portals.
```

### inc:awesome-features (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/inc:awesome-features/source.wikidot.txt:22` through line 30  
SHA-256 of complete source file: `fbf1e000ff4d3e309559744a7d34951bfc3584a1095760458ff6258b4402de16`

```wikidot
L0022       [[div class="col-md-4 feature"]]
L0023         [[div class="icon-sitemap icon-4x feature-icon"]]
L0024         [[/div]]
L0025         [[div class="feature-title"]]
L0026         Working together
L0027         [[/div]]
L0028         [[div class="feature-description"]]
L0029           Work with friends and coworkers on the same documents, share data and files instantly.
L0030         [[/div]]
```

### inc:what-is-wikidot (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/inc:what-is-wikidot/source.wikidot.txt:5` through line 6  
SHA-256 of complete source file: `974584bd77f7a3cd1271181a28b6598aa1bb277fc6417988f1b7330fe8f0d5bf`

```wikidot
L0005 + ##f24747|What is Wikidot?##
L0006 It's simply a place to build wiki-based websites. Use it to publish content, share your documents, collaborate with friends or coworkers, create a place for your community!
```
