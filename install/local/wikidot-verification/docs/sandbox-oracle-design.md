# Driftless sandbox oracle: design

## 1. Why this exists

The standing mirror is validated by comparing against the real, live, third-party
EN corpus at https://scp-wiki.wikidot.com/. That corpus is a moving goalpost: any
Wikidot user can edit a page and any site admin can update a theme at any time,
independent of this project. A comparison that passes today can fail tomorrow for
reasons that are not a local regression, and after the fact there is no cheap way
to tell "we broke something" from "they changed something".

The fix is a corpus we fully control. We author fixture pages on
http://sandbox-for-codex.wikidot.com/ (an owner-controlled sandbox nobody else
edits), capture their live-Wikidot rendering ONCE, and freeze that capture as the
oracle. Because the source pages never change unless we change them, any later
mismatch against the frozen capture is guaranteed to be a real local regression.
This is the same freeze-live-evidence-once discipline already used by the V4
oracle-fixtures system (see src/oracle-fixtures.mjs, "the oracle side is frozen
live evidence - never regenerate it from local output"), extended from context-
free wikitext snippets to full authored pages, which makes the oracle arbitrarily
extensible and re-verifiable instead of harvested-and-static.

sandbox-for-codex already exists as a seeded LOCAL Deepwell site (deepwell/seeder/
sites.json, layout "wikidot"; see tests/sandbox-seed.test.mjs). What is missing is
(1) authorization of the LIVE sandbox-for-codex.wikidot.com side in the mutation
allowlist, (2) the fixture corpus, and (3) the local-vs-frozen comparison wiring.

## 2. Site and authentication

- Live authoring/read-back target: http://sandbox-for-codex.wikidot.com/ (http,
  matching the existing DEFAULT_WIKIDOT_ORIGIN scheme for the JP sandbox).
- Local mirror: https://sandbox-for-codex.wikijump.localhost (https).
- Credentials live out of band at ~/.config/wikijump-oracle/test-account-a.env
  (shell-exportable, WIKIDOT_USERNAME and WIKIDOT_PASSWORD). Do not read, echo,
  or copy the values.
- Authoring routes through the EXISTING helper
  scripts/wikidot_theme_page_helper.py. Mechanics: source the env file, export
  the two vars, invoke the helper. The helper pops the vars from os.environ and
  its reject_secret_fields guard refuses any credential-shaped field passed
  anywhere other than the environment. Do not invent a new auth path.

## 3. Allowlist extension (security-sensitive)

### Current enforcement (do not weaken)

ALLOWED_SITE_SLUG = "scpaiueouiuiuiui" is enforced independently in many layers:
JS validateSiteSlug (strict ===), validateTargetOrigin (builds the expected
hostname from the constant), the Deepwell and Wikidot adapters (each re-checks
and constructs the URL), execution-plan hardAllowlist (site_slug plus both
hostnames), the runner lock filename; and the Python helper independently
hardcodes ALLOWED_SITE plus run-owned-slug regexes. This is a live-Wikidot
MUTATION rail: the toolchain can create, edit, and delete pages on the live site.

### Recommended change

Generalize to a single frozen multi-site allowlist that is the one source of
truth, and migrate every enforcement point to read from it:

- Replace the single constant with a frozen set, e.g.
  ALLOWED_SITE_SLUGS = Object.freeze(new Set(["scpaiueouiuiuiui", "sandbox-for-codex"])).
- validateSiteSlug: membership test against the set.
- validateTargetOrigin: FIRST validate slug membership, THEN build the expected
  hostname FROM THAT VALIDATED SLUG. This is the critical correctness point -
  never validate one slug and construct a URL for another (no confused deputy).
  Origin construction must be parameterized by the same slug that passed the
  membership check.
- Both adapters and the execution-plan hardAllowlist: same membership + build-
  from-validated-slug pattern.
- Python helper: mirror the set. Add a cross-language test asserting the JS
  allowlist and the Python allowlist enumerate the SAME set, so the two copies
  cannot drift.

Reject the alternative of a second single-site constant with a near-duplicate
toolchain: duplicating validation across copies invites the classic rail bug
where one copy is patched and the other is not. For a mutation rail, one audited
source of truth with defense-in-depth membership checks at each layer is safer
than two parallel toolchains.

### Run-owned namespace for oracle pages

Give oracle fixtures their OWN run-owned slug prefix, distinct from the theme-
localization codex-l10n: prefix - e.g. codex-oracle:<runId>-<fixtureId>, with
fixtureId validated against the fixture registry (section 4). This keeps oracle
pages from colliding with or being mistaken for theme-l10n pages, and lets the
helper's ownership check delete/rewrite only pages this system created.

### Sign-off before merge

Widening a live-Wikidot MUTATION rail from one site to two enlarges what the
automated helper (holding write credentials) may create/edit/delete on the public
internet. Even though sandbox-for-codex is authorized in general, the specific
rail-widening diff must be:

1. Its own small, isolated, heavily-tested PR.
2. Reviewed and approved by the owner BEFORE it lands.
3. Never bundled with content/methodology work.

The two items that must stop for sign-off: (1) adding sandbox-for-codex to the
allowed set; (2) any change to validateTargetOrigin or the adapter origin checks.
Design and draft freely; do not merge these without explicit approval.

## 4. Content: the fixture condition matrix

The goal is wide, durable coverage, not a handful of ad hoc pages. Factor the
matrix on two axes that keep it from exploding:

- Syntax DOM shape is theme-independent (FTML renders the same DOM under any
  theme), so syntax fixtures are authored once under one baseline theme.
- Chrome/theme rendering is theme-dependent (skeleton/cascade issues), so chrome
  fixtures repeat one representative page across each canary theme family.
- A small hand-picked set of interaction pages covers known cross-effects.

### Axis 1 - syntax fixtures (one baseline theme, FTML-owned DOM)

Each row isolates a construct family drawn from this engagement's real merged-PR
defect classes, so every fixture guards a concrete prior bug:

- Collapsibles: nested, anchored, crossed, inside quotes/lists (ftml #268, #248,
  #234; wikijump #701).
- Tabview (docs/ftml-boundary.md constructs).
- Footnotes and bibliography: inline footnote, tooltip wrapper, footnote block
  (ftml #213; wikijump #691).
- Native lists: deep nesting, skipped depths, long lists, lists in quotes
  (ftml #252, #253, #675).
- Quotes: tight quotes, contiguous markers, empty rows, quoted includes/code/
  raw/conditional/module markers (ftml #230, #232, #246, #234, #235, #236,
  #237, #238, #258).
- Includes: targetless, quoted, spaced, tight, boundary preservation (ftml
  #250, #241, #227, #229, #222, #217). NOTE: include fixtures need their target
  pages hosted on the sandbox too, run-owned.
- Parser functions (ftml #240).
- Inline formatting: monospace, bold/italic/size/span across block boundaries,
  crossed closers, bracket runs (ftml #266, #282-range, #231, #233, #245, #256,
  #257, #226, #211).
- Images: line-start image paragraphs, image layout, CORS attributes (ftml
  #267, #244; wikijump #685, #684). Needs a real image asset, imported via
  corpus file capture, not added to repo seed data (AGENTS.md).
- Tables: advanced tables, legacy simple tables (ftml #225, #215).
- Code blocks and math.
- Links: scheme-like local page links, anchors (ftml #220).
- Comments [!-- --] visibility during the include scan.
- Document-leading whitespace and paragraph edge cases (ftml #239, #223).

### Axis 2 - chrome/theme fixtures (one page per canary theme family)

Repeat a representative content page under each REQUIRED_THEME_FAMILIES entry
already tracked by the standing promotion gate (src/standing-browser-canaries.mjs):
sigma, basalt, flopstyle-y2k, black-highlighter-calibri. These exercise the
page-chrome skeleton, the top-bar nav/font cascade, the search form, and the
author-card details/div shape. This makes chrome regressions catchable the same
driftless way syntax ones are.

### Interaction pages (small, hand-picked)

Known cross-effects, e.g. a collapsible inside a themed page where theme CSS
historically clobbered collapsible interaction (wikijump #701); a footnote inside
a native list; an image at the start of a quoted block.

### Preserved/delayed constructs - a SEPARATE assertion class

ListPages, CountPages, unknown modules, and conditional blocks render as
PRESERVED/DELAYED structure locally (per docs/ftml-boundary.md) but EXECUTE on
live Wikidot (ListPages returns results). A naive match-live diff would flag a
legitimate divergence. These fixtures use assertion class "match-frozen-preserved":
the oracle side is a hand-declared expected preserved DOM shape (Family-A-style
against a checked-in expectation), NOT the live capture. Do not diff these against
live rendering.

### Fixture metadata (registry entry per page)

Each fixture carries: fixtureId; the construct family it isolates; the PR(s)/
defect-class it guards; the owning surface (FTML vs Framerail/Deepwell); the
assertion class (match-live vs match-frozen-preserved); the theme family (for
Axis 2); and the frozen-capture provenance (datestamp + content hash, e.g.
sandbox-oracle-20260722, following the en-context-free-20260704 convention).

### Provenance and freezing

Author the page, capture its live rendering once via the helper reading it back
from sandbox-for-codex.wikidot.com, freeze it with a datestamp and hash, and never
regenerate from local. Because nobody else edits sandbox-for-codex, the frozen
capture stays valid indefinitely. Intentional page changes require a deliberate
re-author plus re-freeze under a new datestamp.

## 5. Comparison methodology: what "corner to corner" means

Reuse the existing comparison families; do not invent new comparators. Apply
cheapest-first, fail fast at the cheapest layer that catches a regression; a
promotion gate runs all layers.

Layer 1 (cheapest) - DOM signature, exact. src/oracle-fixtures.mjs domSignature/
compareSignatures: exact tag/class/attr counts plus id_count/comment_count over
the page-content fragment, no browser. Fast pre-filter for gross structural drift
and for a details-vs-div count mismatch.

Layer 2 - structural DOM + geometry. src/standing-browser-parity-contract.mjs
compareCaptures: DOM multiset distance over #page-content descendants
(dom_multiset_distance_ratio <= 0.15, tolerant) plus geometry deltas (position
<= 8px, size <= 12px). This is the layout-parity layer.

Layer 3 - computed style and custom properties, exact-normalized. Custom props
off documentElement and computed styles over a defined property set. font-family
is already in COMMON_STYLE_PROPERTIES and the pseudo-layout property set.
Compares against frozen live values with URL/whitespace normalization.

Layer 4 - presence probes and pseudo-layout. Presence probes assert count parity
and rendered-count parity by CSS selector, and the <details>-closed rendered()
special case. Pseudo-layout compares ::before/::after painted bounds
(visible_area_ratio >= 0.95, ratio_delta <= 0.05).

Layer 5 (most expensive) - screenshots. Captured and SHA-256-bound as the human-
facing receipt; there is no pixel diff today, and none is proposed - visual parity
is asserted through geometry plus pseudo-layout painted bounds.

### "Match corner to corner" = Layers 1-4 together

Layer 1 as a fast exact pre-filter, Layer 2 for structure+layout, Layer 3 for
cascade/typography, Layer 4 for presence and pseudo-layout, with Layer 5 as the
visual receipt. A fixture passes the gate only when all applicable layers pass.

### Skeleton-scope gap that MUST be closed

`COMMON_GEOMETRY_SELECTORS` (src/standing-browser-canaries.mjs) is exactly
`["#main-content", "#page-content", "#header", "#side-bar", "#header h1 a"]`.
It does NOT assert the full body -> #skrollr-body -> #container-wrap-wrap ->
... -> #top-bar ANCESTOR chain, and none of those ids appear anywhere in the
selector list. So a missing #skrollr-body, a top-bar rendered as a class instead
of an id, or extra wrapper divs above #container-wrap-wrap are currently
invisible to every existing gate. The oracle must extend capture scope UPWARD to
the page-chrome skeleton - assert the ancestor chain, the expected ids
(#skrollr-body, #container-wrap-wrap, #top-bar), and the wrapper structure - not
just #page-content downward. Without this, a page-chrome skeleton regression
cannot be pinned by anything that exists today.

### Volatile-attribute normalization (required)

Before any structural/attribute comparison, mask a defined set of volatile
attributes: timestamps, CSRF tokens, page IDs, session/request ids, and random
module ids (cf wikijump #640 random ListPages side channel, #683 runtime identity
hash cycle). Reuse render-compare.mjs normalization channels (hostname_map,
request_id, semantic_timestamp, cache_buster, env_id) where they apply. If
enabling a normalization is what makes a difference disappear, that is a finding
(normalization_hides_difference), never a silent free pass.

## 6. What the oracle unblocks

- Pin the page-chrome skeleton, top-bar font cascade, and search-form findings
  as chrome oracle canaries (per theme family), and the author-card details/div
  finding as a fixture, so they can never silently regress.
- Move regression work from drift-prone live-Wikidot eyeballing to oracle-gated
  TDD: new syntax/chrome work adds a red fixture first, then the fix turns it
  green.
- Keep the harvested scp-wiki corpus and the XML-RPC acquisition campaign as the
  DISCOVERY/coverage source (broad but frozen-at-capture); make the authored
  sandbox oracle the PRIMARY driftless regression gate (narrow but re-verifiable
  and extensible).
- Safely expand FTML Layout::Wikidot construct coverage, because each new
  construct gets a deterministic, re-verifiable guard.
