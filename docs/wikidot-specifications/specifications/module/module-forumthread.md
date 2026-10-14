# Forum Thread Module

- Feature ID: `module-forumthread`
- Category: `module`
- Documentation status: `invocation-only`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Recognize and implement the `ForumThread` module at the documented invocation sites. The corpus does not provide a dedicated module reference page.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Live-Wikidot behavioral corrections

The observations in this section are normative and override conflicting or
incomplete documentation-derived evidence below.

### Forum read-only surfaces share category, thread, post, route, and Ajax identities

- Observation ID: `forum-q1034-readonly-route-core-20260809`
- Classification: `documentation-omission`
- Observed at: `2026-08-09`
- Analysis: Anonymous PagePreviewModule, served GET, and Ajax Module Connector observations establish one read-only baseline over existing forum data. They define the default FrontForum item shape, public forum routes, exact module names and scalar parameters for page comments, forum start, category, thread, thread posts, and recent posts, the complete first-page thread envelope, and the exact category order control and pager on pages 1, 2, 11, and 12. They do not establish mutation authority, non-anonymous actor behavior, FrontForum custom formats, later Comments or RecentPosts pages, order-link navigation or sorted behavior, or browser transitions.

Normative behavior:

- FrontForum with a double-quoted positive category ID and an observed positive limit renders newest threads from that category in a front-forum-box. Each item uses the first post as content and reports comments as the active post count minus one. A nonexistent category renders Requested forum category does not exist.
- forum/ForumCommentsListModule accepts the observed pageId request and the same request with order=reverse. It maps the page to threadId and returns the first comments page with the observed thread script, comments options, hidden thread-container-posts, duplicate pager, ten root comment trees and their descendants, new-comment control, and post-options template. Forward selects the oldest ten roots and orders their trees forward; reverse selects the newest ten roots and reverses root order while retaining descendants under their parent. The pager counts root comments. The forward jsInclude order is ForumViewThreadModule.js, ForumViewThreadPostsModule.js, ForumNewPostFormModule.js; reverse places ForumNewPostFormModule.js before ForumViewThreadPostsModule.js. These remote URLs are observed response metadata and do not establish loader or mutation authority. A nonexistent page returns no_page.
- forum/ForumStartModule accepts the observed absent parameters or hidden=true. Its read model includes category counts and last-post identity; the ordinary view hides hidden groups and hidden=true includes them.
- forum/ForumViewCategoryModule uses c and p, returns status no_category for a nonexistent category, and renders the category summary and ordered thread table for existing data. The observed populated category uses 20 thread rows per page; a final partial page has one row, and an out-of-range page has no thread rows.
- forum/ForumViewThreadModule uses t and returns status no_thread for a nonexistent thread. Its body owns forum-thread-box, options, hidden thread-container-posts, first-page post-container roots, post templates, edited metadata, and the sibling thread-id script. forum/ForumViewThreadPostsModule requires t and first-page pageNo and returns sibling post-container roots without an outer thread-container.
- The thread response jsInclude order is ForumViewThreadPostsModule.js then ForumViewThreadModule.js; the posts response lists only ForumViewThreadPostsModule.js. These remote URLs are observed response metadata and do not establish loader authority.
- forum/ForumRecentPostsListModule uses page and categoryId. The empty category selects all visible categories, an existing category narrows the result, and a nonexistent numeric category returns status ok with an empty post container.
- The observed public GET routes are /forum/start, /forum/start/hidden/show, /forum/c-<id>/<name>, /forum/c-<id>/p/<page>, and /forum/t-<id>/<name>. Missing category and thread routes remain successful page responses with the observed error text; other suffixes are not established.
- Immediately after the category description and before the new-thread control, pager, and thread table, every observed category page renders the exact default order projection <div class="options">Order by: <div class="btn btn-primary disabled btn-small btn-sm"><strong>Last post date</strong></div> <a href="/forum/c-<category-id>/sort/start" class="btn btn-primary btn-small btn-sm">Thread starting date</a></div>. This establishes the retained DOM and dynamic category ID only; it does not establish /sort/start navigation, sorted behavior, or the new-thread control.
- The category response has duplicate top and bottom pagers on full pages and one top pager on final partial and out-of-range pages. Page 1 links 1, 2, 3, ..., 10, 11 and next 2; page 2 links previous 1, 1, 2, 3, 4, ..., 10, 11 and next 3; page 11 links previous 10, 1, 2, ..., 9, 10, 11; page 12 deliberately says page 12 of 11 and links previous 11, 1, 2, ..., 10, 11, 12, and next 13. Pager hrefs use /forum/c-<id>/p/<page>.
- All captured requests are anonymous and read-only. Observed controls and templates do not establish Comments, new-thread, post, edit, delete, lock, move, or other mutation authority.

Raw HTTP captures:

- scp-category-1113520-page-1 body stored locale `en`: `/home/roku/wjlab/evidence/20260808-open87-execution/pr2-b21a91941/forum-g9-pagination/raw/category-page-1.html` (SHA-256 `5910330cc219f216925076a661019a8f624aada45629606c9f3b4bdde579a247`)
- scp-category-1113520-page-2 body stored locale `en`: `/home/roku/wjlab/evidence/20260808-open87-execution/pr2-b21a91941/forum-g9-pagination/raw/category-page-2.html` (SHA-256 `d43aa71385e23b7670ff9aba13afda2fa2d58e5ae93b647bb32cbb62bd37ef50`)
- scp-category-1113520-page-11 body stored locale `en`: `/home/roku/wjlab/evidence/20260808-open87-execution/pr2-b21a91941/forum-g9-pagination/raw/category-page-11.html` (SHA-256 `e97127cf83883f3938cafa65182549174762b0a470d6ae4f875eda902a198d7c`)
- scp-category-1113520-page-12 body stored locale `en`: `/home/roku/wjlab/evidence/20260808-open87-execution/pr2-b21a91941/forum-g9-pagination/raw/category-page-12.html` (SHA-256 `4a7207b4d90c8009570ee284fd9fd8f77a6193b26f0b85d7df22d5ee46a0134a`)

Evidence:

- `install/local/wikidot-verification/artifacts/forum-q1034-readonly-live-20260809.json` (SHA-256 `0a188e7960890a0ad05fbb7733671072abc1f08156e0d2df7e70b523e3405fd4`), cases: `frontforum-sandbox-limit-one`, `frontforum-sandbox-limit-two`, `frontforum-sandbox-invalid-category`, `frontforum-scp-limit-one`, `sandbox-forum-start-visible`, `sandbox-forum-start-hidden`, `sandbox-category-populated-page-one`, `sandbox-category-empty-page-one`, `sandbox-category-missing`, `sandbox-thread-populated`, `sandbox-thread-missing`, `sandbox-thread-posts-populated`, `sandbox-thread-posts-missing`, `scp-comments-forward`, `scp-comments-reverse`, `scp-comments-missing-page`, `sandbox-recent-posts-all-page-one`, `sandbox-recent-posts-category-page-one`, `sandbox-recent-posts-missing-category`
- `install/local/wikidot-verification/artifacts/forum-q1034-pagination-live-20260809.json` (SHA-256 `48c014f29e3ffa893073ef90048b353880d929e2bb612d358ee977dafbe679b2`), cases: `scp-category-1113520-page-1`, `scp-category-1113520-page-2`, `scp-category-1113520-page-11`, `scp-category-1113520-page-12`


### Reply visibility follows effective nesting and sibling position

A controlled anonymous observation on 2026-10-12 varied only a run-owned
category's maximum nesting level around the same stored root and two sibling
replies. Both ForumViewThreadModule and ForumViewThreadPostsModule agree:
level 0 hides every Reply control; below the effective maximum, Reply is
shown; at or above the maximum, only the final sibling shows Reply. Lowering the
setting preserves existing deeper trees and their final-sibling controls. Resolve the
category override through the site's default when absent. This is a
navigation control and does not grant posting authority.

Frozen property matrix with raw-response digests:
`deepwell/tests/fixtures/forum-reply-nesting-live-20261012.json`.
The original group/category settings were restored and run-owned posts
removed. Empty thread tombstones remain in the sandbox's Deleted threads
category, as recorded in the retained cleanup evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- The documentation corpus proves the module name and invocation context, but not a complete behavior contract.
- Before implementing behavior beyond the recorded invocation, capture live Wikidot output at the public rendering or browser seam and add that evidence to this specification.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/forum:thread/source.wikidot.txt:1` through line 1 (invocation-only)

## Documentation-derived behavioral evidence

### forum:thread (invocation-only)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/forum:thread/source.wikidot.txt:1` through line 1  
SHA-256 of complete source file: `ba4849548772e5afcb099718d64d0b6427799827ef9fd6761dc01007b0034445`

```wikidot
L0001 [[module ForumThread]]
```
