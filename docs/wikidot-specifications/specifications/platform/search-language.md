# Search query language

- Feature ID: `search-language`
- Category: `platform`
- Documentation status: `documented`
- Detailed conformance status: `detailed-p1-p8`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement Wikidot's basic, filtered, global, and tag-oriented search behavior and query syntax.

## Implementation contract

- The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.
- Account, site, category, page, and actor context MUST be enforced at the public service boundary.
- Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Detailed conformance contract

- Status: `detailed-p1-p8`
- Source-gap snapshot: Wikijump `257f6a3936976f1a6ea5094ae0cee5ac12777495`
- Evidence manifest: `docs/wikidot-specifications/detailed-spec-evidence-20260816.json`

This section is normative. It maps the complete evidence below to every P1-P8
implementation axis. A statement that deliberately keeps an unobserved path
fail-closed is a boundary of the specification, not permission to invent the
missing Wikidot behavior.

Evidence basis:

- `current-www-source` -> `/home/roku/wjlab/evidence/spec-hardening-20260816/live-www-source-pages.jsonl` (SHA-256 `53ffba0adb068777ad023eb46dabb59756223fc13ab10d7c9b4a82042b276ffc`): All 46 current www.wikidot.com source pages referenced by the 57 hardened features were found and all 46 source hashes matched the frozen documentation corpus.
- `canonical-live-observations` -> `scripts/data/wikidot-live-observations.json` (SHA-256 `accbc415e44604acc39c2194204da975203acf6baf7ceb114e02835907c1d686`)

### P1 - invocation grammar and scalar interpretation

- Basic search accepts ordinary terms, +required terms, title: terms/quoted phrases, wildcards, category filters, since/till dates, user filters, global site filters, and the documented tags filter syntax. Tags search is documented as disabled for performance unless live service evidence shows otherwise.

### P2 - parser stage, nesting, and composition

- Search tokens are parsed by the search query language, not by wiki-source parsing. Quoted phrases, comma lists, wildcard placement, and filter prefixes are compatibility-sensitive.

### P3 - lifecycle, persistence, import, and round trips

- Search indexes derive from current site/page state but search itself does not mutate pages. Index delay/rebuild must not change authoritative page content.

### P4 - actors, permissions, visibility, and privacy

- Site/global results MUST filter by the requesting actor before exposing private-site/page matches; global search may include private sites only when the actor is allowed to read them.

### P5 - selection, ordering, counting, and pagination

- Pages are ranked by matching search terms and filters reduce the result set. Pagination/order must be applied after permission filtering, not before.

### P6 - HTTP, API, URL, Ajax, feed, and navigation contracts

- Every site header exposes a site search form; global search uses www.wikidot.com/search:all. Current live global SearchAll evidence shows the form works but non-empty global queries can return an ElasticSearch-down error; a backend outage is a valid observed service state, not permission to fabricate results.

### P7 - DOM, CSS, resources, interaction, and geometry

- Search forms/results/errors must preserve Wikidot's public form and result/error DOM for the observed route. Query text must be safely encoded in navigation and rendered output.

### P8 - temporal behavior, failure atomicity, limits, and resource bounds

- Unavailable search backend must fail as a search error without leaking private index data or hanging page navigation. Wildcard/quoted input must be bounded against pathological query expansion.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:searching/source.wikidot.txt:1` through line 32 (canonical)

## Documentation-derived behavioral evidence

### doc:searching (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:searching/source.wikidot.txt:1` through line 32  
SHA-256 of complete source file: `54ad518d21b23d14b16b2cae9e97f1439f440487a25e1e2d6b331eecbc94ddb5`

```wikidot
L0001 ++ Basic searching
L0002 
L0003 Every site header shows a search form that lets you find pages in that site.  In the search form, type the words you want to search for and press Enter.
L0004 
L0005 * Pages are ranked on how many search terms they match, so to refine your search results, add more words.
L0006 * To show //only// pages that contain //elephant//, search for **+elephant**.
L0007 * To show pages that have //elephant// in the title, search for **title:elephant**.
L0008 * To show pages that have the phrase //grey elephants// in the title, search for **title:"grey elephants"** using double quotes.
L0009 * To show pages that contain any word starting with //ele//, search for **@@ele*@@**.
L0010 * To show pages that contain any word starting with //ele// and ending in //ant//, search for **@@ele*ant@@**.
L0011 
L0012 ++ Additional filters
L0013 
L0014 * To restrict the search to category "abc", add the filter **category:abc**.
L0015 * To restrict the search to categories "abc" and "def", add the filter **category:abc,def**.
L0016 * To restrict the search by date, add either or both of: **since:yyyy-mm-dd** and **till:yyyy-mm-dd**.
L0017 * To restrict the search to a specific author, add **user:author-name**, using '-' or '_' instead of spaces in the user name.
L0018 
L0019 ++ Global searching
L0020 
L0021 You can search all of Wikidot, including private sites that you are allowed to read, at http://www.wikidot.com/search:all.  Global search works like basic searching, with these additional options:
L0022 
L0023 * To show pages in the site elephants.wikidot.com, search for **site:elephants**.
L0024 * To show pages in several specific sites, list these with commas after the **site:** keyword.
L0025 
L0026 ++ Searching on tags
L0027 
L0028 //Note: searching on tags is currently not enabled for performance reasons.  When we've solved those, we'll enable tag searching.//
L0029 
L0030 In basic and global searches you can additionally search for pages that have specific tags:
L0031 
L0032 * To show pages that contain the tags "big" and "noisy", search for **tags:big,noisy**.
```
