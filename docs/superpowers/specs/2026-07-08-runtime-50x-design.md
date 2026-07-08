# Runtime 50x design: fast-import V2 pipeline and page-serving latency

Date: 2026-07-08
Status: approved direction, pending implementation planning
Scope: local Wikijump lab runtime (docker compose project class `parityint*`), EN corpus site 6000006

## Targets

T1 (import). Wall-clock time from corpus to fully materialized site: all pages
registered, all attachments materialized, all pages rendered with real compiled
output. Baseline: P0 sequential RPC import at 2,285 pages/hour
(`evidence/p0/pilot-report.md:27`), which projects to ~10.7 hours for the
24,430-row EN corpus. Committed milestone: ~21-25 minutes end-to-end (26-31x).
Follow-up slices target ~13 minutes (50x).

T2 (serving). Server response time for an article request by slug (HTTP request
to rendered HTML response). Baseline: ~60 ms median measured on the current
debug/dev stack for `scp-173` (46-130 ms range over 10 requests). Target:
~1.2 ms for hot anonymous cache hits (50x), 10-20 ms for cold or authenticated
views. Cache hits count toward the target; staleness is excluded by
construction (revision-keyed cache, not TTL-based).

Accounting decisions (user-approved 2026-07-08): approach A (pipelined direct
materialization); import 50x staged as 30x committed + 50x follow-up; serving
50x measured on server response with cache hits allowed.

## Baselines with provenance

| Number | Measures | Source |
|---|---|---|
| 2,285 pages/hour | P0 sequential RPC full import | `evidence/p0/pilot-report.md:27` |
| 78.9-118.5 rows/hour | A3 RPC-mode full import (abandoned) | `evidence/a3-full-import/a3-calibration-stop-20260707.json` |
| 24,430 rows in 11m57s (~122,661 rows/hour) | V1 shell registration, attachments deferred | `evidence/v1-fast-import/pr280-merge-reconciliation-20260708.md:30-45` |
| ~18,064 attachments/hour | Serial RPC attachment materializer (pilot, 1,142 files in 227.6 s) | `evidence/p2/pilot-attach-batches/` |
| 50,220 attachments, 47,677 unique sha256, 23.1 GB | Full EN attachment inventory | `evidence/p2/en-full-manifest-attachments.jsonl` |
| ~60 ms median server response | `scp-173` on debug deepwell + Vite framerail + sqlx logging on | measured 2026-07-08 (research run) |

Evidence root: `/home/roku/codex-thread-workspaces/local-wikidot-lab-20260706/evidence/`.

## Key findings the design rests on

1. Page views never render at view time. Deepwell serves stored compiled HTML
   from `text` rows referenced by `page_revision`
   (`deepwell/src/services/view/service.rs:246-285`). Therefore T2 is a
   read-path problem, and V1 shell pages serve placeholder stub HTML until
   re-rendered, which makes the render finalizer load-bearing for both targets.
2. V1 shell import writes real `page`, `page_revision`, `text`, and snapshot
   rows but leaves undone: real compiled body, nav hashes, link graph,
   outdater, text blocks, and files
   (`apply-corpus-import-manifest.mjs:393-540`).
3. `wikidot_corpus_import_item` already has state, attempts, and lease columns
   (`deepwell/migrations/20260625104500_wikidot_corpus_import.sql:50-80`) and
   is the natural durable queue for render finalization.
4. The existing Redis job queue defaults to 2 workers, has no dedupe and no
   import-state integration; it is the wrong tool for bulk finalization.
5. The RPC attachment path pays a double-S3 tax (operator PUT to temp key,
   deepwell GET + re-PUT at SHA-512 key, delete temp) plus 3 JSON-RPC calls per
   file (`deepwell/src/services/blob/service.rs:337-456`).
6. Local stack overhead is real: deepwell runs as a debug `cargo watch` build
   (`install/local/deepwell/deepwell-start:12-18`), framerail runs the Vite dev
   server (`install/local/framerail/Dockerfile:20-21`), sqlx statement logging
   is hardcoded on (`deepwell/src/database/mod.rs:35-40`), Postgres is stock
   `postgres:17-alpine`.
7. Include expansion reads latest wikitext of the included page, not compiled
   HTML (`deepwell/src/services/render/service.rs:2870-3072`), so bulk render
   needs no topological ordering; only parent links must land first, and
   Backlinks-module pages need a second pass after the link graph exists.
8. `TextService::create` is exists-then-insert keyed on hash
   (`deepwell/src/services/text.rs:167-185`); parallel renders of identical
   output race on `text.hash` and need insert-on-conflict.
9. File serving invariants (wws): the S3 object must exist at exactly the
   SHA-512 key; `mime`, `size`, and `s3_hash` on the DB row drive
   browser-visible headers (`wws/src/fetch.rs`, `wws/src/handler/file.rs`).

## Workstream 1: stack baseline (shared multiplier)

Three independent slices.

1a. Make sqlx logging configurable: `[database] sqlx-logging` in deepwell
config, default true (unchanged for normal deployments), off in the lab
config. Touches `deepwell/src/database/mod.rs` and config plumbing.

1b. Local compose toggles for a release deepwell (either
`cargo run --release` in the start script or the dev/prod release-image
pattern) and a built framerail (node adapter instead of Vite dev). Opt-in via
compose/env so the default local dev loop is unchanged.

1c. Lab-only Postgres relaxations: compose override with
`synchronous_commit=off`, `fsync=off`, `full_page_writes=off`. Fenced to the
disposable local compose only (volumes are anonymous and reset on container
recreate; durability loss is acceptable there by construction). Documented in
the override file itself.

Expected effect: 2-5x on CPU-heavy render paths, 1.05-1.5x from logging,
1.2-3x on commit-heavy bulk writes. Applies to both T1 and T2.

Evidence: fixed page-view latency sample (curl percentiles on a pinned page
set) and a shell-registration rerun, before/after each slice.

## Workstream 2: attachment direct materializer

Extend `apply-corpus-import-manifest.mjs` with `--attachment-create-mode
direct` (requires `--db-url`, reuses the V1 persistent pg executor from
`install/local/wikidot-verification/src/corpus-import-sql.mjs`).

Phase 1 (blobs): verify corpus sha256, compute SHA-512 and MIME locally,
upload unique blobs directly to MinIO at their content-addressed SHA-512 keys
with bounded concurrency (32-64 workers), HEAD-first idempotency, size
verification after PUT.

Phase 2 (rows): staging table keyed by (site_id, page_slug, filename, sha256,
sha512, size, mime, file_path); bulk insert `file` and `file_revision`
(revision_number 0, create semantics) by joining pages; per-row match/skip/
fail: existing rows that match size+hash are skipped, mismatches fail closed
and are reported. JSONL per-row results and summary fields identical in shape
to the RPC path. Dry-run supported.

Deliberate deviations from the RPC path, with rationale:

- No page outdating per file: every page is re-rendered by workstream 3
  anyway.
- No temp-key round trip: hashes are computed locally; correctness is enforced
  by the serving invariants below and proven differentially.

Invariants enforced in phase 2: S3 object exists at the exact SHA-512 key with
matching size; `mime` and `size` match served content; blob not blacklisted;
active (page_id, name) uniqueness; first revision shape matches deepwell's.

Validation: differential test on a pilot subset (RPC path vs direct path must
produce equivalent `file`/`file_revision` rows and identical served bytes),
then wws serving checks and the parity validators on attachment-bearing pages.

Expected effect: 2.8 hours serial -> 10-20 minutes, overlappable with shell
registration and rendering. Floor is MinIO upload bandwidth for 23.1 GB.

## Workstream 3: render finalizer

A batch finalizer inside deepwell as a new module plus subcommand (not growth
in `render/service.rs`), e.g. `deepwell render-finalize`.

Queue semantics: claim `render_pending` rows from `wikidot_corpus_import_item`
using the existing lease/attempt columns; set `render_running`; on success
mark `rendered`/`done`; on failure `render_failed` with attempts respected.
Fully resumable; same-page duplicates are no-ops via per-page lease.

Render semantics: call the existing `PageRevisionService::rerender` path so
includes, ListPages/CountPages, nav rendering, text blocks, and
`LinkService::update` all run through product code. No reimplementation of
render logic outside deepwell (a Node+FTML shortcut is explicitly rejected:
normal render is far more than FTML and would silently diverge).

Concurrency: bounded tokio concurrency (configurable, start at 16), shared
connection pool. Prerequisite fix: `TextService::create` becomes
insert-on-conflict so parallel identical outputs cannot race on `text.hash`.

Pass structure:

- Pass 1: render all shell pages with the outdater suppressed (otherwise the
  corpus redundantly queues itself). Runs only after `upsertParentLinks` so
  parent-based ListPages selectors see `page_parent`.
- Pass 2: targeted rerender of dependency-sensitive pages (Backlinks-module
  pages, nav pages, template pages) after the link graph is populated by
  pass 1.

Budget: 0.2-0.35 s/page at 16 workers on a release build gives 5-9 minutes for
24,430 pages.

Validation: V2 render-health sweep against existing pilot verdicts
(`evidence/p0/v2-pilot-rerun/verdict.json`, `evidence/p2/pilot-v2-post-attach/
verdict.json`) plus render regression canaries per the repo rules for render
code.

## Workstream 4: serving fast path

4a. Consolidated article-view RPC: one deepwell endpoint returning preload
viewer data, page view data, and translation results in a single transaction;
framerail's `[slug]` route makes one RPC instead of three sequential ones
(`framerail/src/lib/server/load/page.ts:75-93`). Removes duplicate
site/session reads and two transaction commits. Expected 10-20 ms after
workstream 1.

4b. Revision-keyed anonymous response cache: Redis (already in the stack),
key = (site_id, slug, extra-path, latest_revision_id, nav hashes, locale,
anonymous flag), value = the consolidated RPC read model, later optionally the
full HTML response. Because the key contains the revision id and nav hashes,
edits and nav changes miss naturally; stale content is structurally
impossible rather than TTL-bounded. Expected 3-8 ms hot; the full-HTML variant
reaches ~1.2 ms and closes T2's 50x.

4c (out of scope unless 4b proves insufficient): caddy-level SSR bypass
serving cached full HTML for anonymous imported pages. Highest parity and
invalidation risk; revisit only with 4b evidence in hand.

Parity rule: a cache hit must be byte-identical to the uncached response for
the same revision; the latency harness checks this on every measured sample.

## PR staging

Nine slices, each within the repo's change-size guidance, in dependency
order:

1. sqlx logging config switch (tiny; deepwell config + database/mod.rs).
2. Local compose release/built toggles for deepwell and framerail (small,
   infra only).
3. Lab-fenced Postgres tuning override (tiny, infra only).
4. `TextService::create` insert-on-conflict (tiny; unblocks parallel render).
5. Attachment direct materializer (medium, Node; apply script + staging SQL).
6. Render finalizer core: claim loop + pass 1 (medium, Rust, new module).
7. Render finalizer pass 2 + health reporting (small, Rust).
8. Consolidated article-view RPC (medium; deepwell endpoint + framerail
   call-site).
9. Framerail/Redis response cache (medium).

Follow-up 50x-closer slices for T1, measured before commitment: batched
multi-row shell inserts in the apply script, faster text-hash helper
(release-built), possibly overlapping shell registration with blob upload.

## End-to-end runbook shape (30x milestone)

Pipelined full-corpus run: start attachment phase 1 (blob upload) immediately;
shell registration (~12 min) in parallel; attachment phase 2 after shells;
render finalizer pass 1 after parent links; pass 2 after pass 1. Critical path
is roughly shells (12 min) + render (5-9 min) with attachments fully
overlapped: ~21-25 minutes total, versus the 10.7-hour baseline.

## Error handling

Every fast path is fail-closed per row with durable state and JSONL evidence:
attachment mismatches fail the row and report; render failures mark
`render_failed` with attempts; cache misses fall through to the normal path.
No fast path ever widens behavior silently; unsupported shapes stay on the
slow/normal path.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Render divergence from product path | Only deepwell service code renders; Node/FTML shortcut rejected |
| Outdater storm during bulk render | Suppressed in pass 1; organic outdating resumes afterward |
| Backlinks/nav correctness | Dedicated pass 2 after link graph population |
| text.hash race under parallel render | Insert-on-conflict fix (slice 4) before finalizer lands |
| Cache staleness | Revision-keyed cache keys; byte-equality check in harness |
| Postgres tuning blast radius | Compose override fenced to disposable local lab only |
| Debug-vs-release behavior drift | Same code, same tests; parity canaries rerun on release stack |
| Session expiry mid-run | Direct paths use `--db-url`, no RPC sessions on bulk paths |

## Validation expectations

Narrow first, then broaden, per repo rules: focused Rust tests for the
finalizer module and text-service fix; node tests for the materializer;
differential pilot test RPC-vs-direct for attachments; V2 render-health sweeps
against prior verdicts; fresh browser evidence via the wikidot-verification
tooling for parity claims; latency harness (curl percentiles, pinned page set)
for T2 numbers. Local output is never its own oracle: parity is always judged
against Wikidot evidence with provenance.
