# FTML/Wikijump responsibility boundary

This document freezes the working responsibility boundary between the FTML library (`https://github.com/Rokurolize/ftml`, consumed by Deepwell via `deepwell/Cargo.toml`) and the Wikijump runtime. It is the Stage 0 deliverable of the boundary audit recorded at `codex-thread-workspaces/local-wikidot-lab-20260706/evidence/ftml-wikijump-boundary-audit-20260709/` (local evidence root, not in this repository). Finding IDs like BND-05 below refer to that audit.

This is a contract document, not a migration plan. Nothing here changes runtime behavior. Migration happens in later staged PRs, each gated on evidence.

## FTML-owned responsibilities

- Tokenization, parsing, and AST representation for Wikidot/FTML syntax, including malformed-but-real Wikidot shapes that appear in corpus evidence.
- Syntax-level HTML rendering for both `Layout::Wikijump` and `Layout::Wikidot`, including DOM shape (classes and structure) for constructs such as tabview, footnotes, bibliography, collapsible, code, math, and typography.
- Escaping and sanitization at the syntax render boundary.
- Wikidot comment semantics (`[!-- --]`) everywhere they matter, including the visibility of `[[include]]` and other tokens inside comments during the include scan.
- Structured preserved/delayed representations for runtime constructs: includes, ListPages, CountPages, unknown modules, and conditional blocks such as iftags.
- The parser performance envelope: dense or large real corpus pages must parse within production budget, or fail in a structured way the caller can detect and turn into a fail-closed literal render.
- Reusable syntax fixtures: `test/<group>/<case>/` with `wikidot.html` parity assertions, and article-driven regressions under `tests/*_wikidot_syntax.rs`.

## Wikijump-owned responsibilities

- Site/page/user/file lookup, permissions, DB queries, import/corpus state, and source provenance.
- Include source fetching, cross-site mapping, recursion and total-count limits, and missing-include policy.
- ListPages/CountPages selector semantics, query execution, pagination, permission filtering, and runtime variable values.
- Module runtime data (backlinks, tag cloud, members, rate scores) and placeholder policy for unsupported modules.
- Local file URL localization and materialization, theme chrome, styleFrame runtime behavior, WWS/Caddy routing, and CSP configuration.
- Application chrome, Framerail behavior, browser capture, V2/V3 validators, and the oracle harness.

These runtime responsibilities stay in Wikijump permanently. ListPages/CountPages execution, permissions, DB state, source provenance, files, browser capture, and V3 comparison are never candidates for migration into FTML, even when the surrounding code looks syntax-related.

## Split ownership patterns

Some constructs need both layers. The pattern is always: FTML parses and represents; Wikijump resolves with runtime context.

- Includes: FTML scans (comment-aware once BND-05 lands) and represents `IncludeRef` plus variables; Wikijump supplies page sources and applies runtime policy.
- ListPages/CountPages: FTML preserves the module node with raw arguments and body; Wikijump consumes the preserved node (target state; today it regex-parses raw source) for selector parsing and execution.
- iftags and `[[#if]]`: target state is an FTML delayed conditional node with Wikijump supplying tag/expression truth; today Wikijump owns both halves textually.
- Runtime lookups during render (page titles, page existence, file URLs, user info, module output): target state is an injectable FTML handle implemented by Wikijump. Today `ftml::render::Handle` is a stub, which is the root enabler of the pre-splice/post-rewrite architecture documented below. Any handle API design requires an explicit architecture review before implementation.

## Policy/provenance/lab-owned responsibilities

Golden pairs, the accepted-diff ledger, source-freshness policy, capture-visible-text scope, the V2 failure taxonomy, and the D-decision gates belong to the Local Lab validation layer. They live in lab evidence roots and `install/local/wikidot-verification/`, not in FTML and not in Deepwell render code. The parser-like scanners inside verification tooling (`install/local/wikidot-verification/src/dependency-closure.mjs`, `src/render-health.mjs`, `src/oracle-fixtures.mjs`) are classifier approximations by design; they are validators, not render truth, and must keep their non-oracle labeling.

## Do-not-move-to-FTML list

The following remain in Wikijump even though they touch syntax-shaped text. The full annotated list with code paths is in the audit's `do-not-move-to-ftml.md`.

- ListPages/CountPages query execution, selector semantics, pagination, permission filtering, and runtime variable substitution against page rows.
- Include source fetching, cross-site resolution, recursion/total limits, and missing-include policy.
- Local file URL localization/materialization, asset mirror routing, and external CSS dependency admission through Framerail's CSP allowlist.
- All permission filtering during render, user display resolution, and actor state.
- Module runtime output and unsupported-module placeholder policy (a Local Lab D6 user decision, not an FTML question).
- Iframe allow-listing and interwiki URL rewriting (network/site policy); only the `[[embed]]` syntax parsing half is migration-eligible.
- Theme chrome ordering, shell compatibility, styleFrame runtime, Framerail CSP, WWS html-block wrappers and support-resource routing.
- Render orchestration, operational timeouts, task spawning, text-block storage.
- Sentinel protection for Wikijump's own runtime-generated trusted HTML fragments.
- Everything in the lab tooling layer above.

## Current temporary drift/debt surfaces

The audit found the syntax half of this boundary inverted: `deepwell/src/services/render/service.rs` currently contains three hand-rolled renderers plus a second HTML sanitizer, all of which are debt to be paid down through the FTML backlog, not extended.

- The oversized Wikidot compatibility fallback renderer and its sanitizer (BND-01), triggered by heuristics including a page-name-specific condition (BND-02).
- Post-render DOM rewriting of FTML output into Wikidot DOM for tabview, footnotes, collapsible, code, and math (BND-03).
- Residual marker restorers that re-parse rendered HTML for leftover div/span/alignment/separator/heading markers (BND-04).
- Comment masking around `ftml::include` because the FTML include scan is not comment-aware (BND-05).
- Pre-FTML inline protectors for color spans, escaped entities, bibcite closers, wikipedia links, anchors, current-page links, star local links, multiline page links, and embed iframes (BND-08).
- Native-list pre-rendering and the ListPages-body inline mini-renderer (BND-09).
- Textual iftags evaluation scattered across pipeline phases (BND-10).

These surfaces are frozen: they may receive correctness fixes, but they must not grow new FTML-candidate capability without the deviation-note process below.

## Marker protocol and pin-bump canary requirement

FTML main has begun consuming canonical Wikidot source markers at parse time (ftml PR #178, section markers), while this repository carries HTML-side restorers for overlapping marker classes (`restore_residual_wikidot_*` in `deepwell/src/services/render/service.rs`). The two mechanisms must not silently double-handle or orphan each other.

Rules:

- Before any FTML pin bump (any change to the `ftml` entry in `deepwell/Cargo.toml` or its `Cargo.lock` revision), run a marker-contract canary: render a golden-pair subset with the bumped FTML in a throwaway worktree and compare V3 visible text against current results, specifically covering heading, separator, div, span, and alignment marker surfaces.
- A pin bump PR must link the canary evidence. No canary, no bump.
- After a bump, audit the restorer/protector inventory for passes that became dead or double-handling, and remove them only in separate follow-up PRs with their own golden-pair evidence.

## The wj-* DOM contract caveat

`ftml/README.md` states that `wj-` prefixed classes are generated output not intended for direct use. This repository nonetheless regex-matches exact `wj-code`, `wj-tabs`, `wj-footnote`, and `wj-math` DOM shapes in `deepwell/src/services/render/service.rs` to rewrite them into Wikidot DOM (BND-11). This is an undocumented dependency that FTML has never promised to keep stable.

Until the corresponding `Layout::Wikidot` renderer branches exist in FTML, treat the exact `wj-*` shapes consumed by these rewrites as a temporary informal contract: FTML-side changes to these shapes are expected to break Wikijump restorers, and the pin-bump canary above is the detection mechanism. The durable fix is FTML-side Wikidot layout branches pinned by `wikidot.html` parity fixtures, after which the restorers and this caveat are deleted.

## Rules for new syntax-level shims in Deepwell render code

- No new FTML-candidate syntax shim (parsing, protecting, restoring, normalizing, or rendering Wikidot syntax) may land in `deepwell/src/services/render/service.rs` or sibling render modules without a deviation note using the template below, committed in the same PR.
- The deviation note must include an explicit FTML backlog decision: either a reference to an existing FTML backlog item that covers the gap, or a statement of why the gap is being newly accepted as Wikijump-side debt.
- Runtime behavior (ListPages/CountPages execution, permissions, DB state, source provenance, files, browser capture, V3 comparison) stays in Wikijump and needs no deviation note; the note requirement applies only to syntax-layer work.
- Corpus-specific compatibility workarounds must be labeled as corpus policy in the note; if one later turns out to be general Wikidot grammar, reclassify it explicitly before extending it.

### Deviation note template

Deviation notes live in `docs/ftml-boundary-deviations/` as one Markdown file per shim, named after the primary function or pass.

```markdown
# Deviation: <shim name>

- Shim: <function/pass names and file:line>
- Reason it lives in Wikijump: <one or two sentences>
- Why FTML is not yet sufficient: <missing capability, with FTML paths if known>
- Evidence: <exact evidence paths (lab evidence root, PR, oracle consultation)>
- FTML backlog decision: <existing backlog item reference, or why new debt is accepted>
- Migration condition: <what must be true in FTML before this shim shrinks>
- Owner: <who tracks it>
- Review trigger: <pin bump, date, or event that forces re-evaluation>
```

Shims that existed before this document are inventoried in the audit's `candidate-wikijump-cleanup-backlog.md` and do not require retroactive notes; they are covered by the frozen-surfaces rule above.

## First code migration pilot: BND-05 (recorded, not implemented)

The first code slice after this document is the FTML comment-aware include scan. It is the smallest well-evidenced boundary fix and pilots the full migration pipeline:

1. Add FTML fixtures/tests for comments containing include syntax (include-inside-comment, comment-inside-include-arguments, unterminated comment before include), following the existing `tests/*_wikidot_syntax.rs` pattern.
2. Implement FTML include scanning that ignores Wikidot comment blocks.
3. Bump the Wikijump FTML pin only after the marker-contract canary above passes.
4. Remove or reduce the Wikijump masking shim (`mask_wikidot_comment_include_markers` / `unmask_wikidot_comment_include_markers`) only after golden-pair evidence proves equivalence.

Migration risk is low because Wikijump already guarantees FTML never sees comment-hidden includes, so the FTML-side change is behavior-compatible for Wikijump and strictly safer for other callers.

## Follow-up

- Add a small FTML-side pointer to this document in the FTML repository's documentation surface (kept out of this PR to avoid a forced cross-repo change).
- Subsequent stages (FTML fixtures, FTML implementation slices, Wikijump shim reduction, Local Lab regression verification) are sequenced in the audit's `next-priority-map.md`.
