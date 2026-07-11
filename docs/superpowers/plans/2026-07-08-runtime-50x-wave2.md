# Runtime 50x Wave 2 Implementation Plan

Date: 2026-07-08
Status: planned after wave 1 merge
Scope: runtime 50x slices 5-7, split so no implementation PR needs to carry the whole attachment materializer or render finalizer at once.

## Goal

Wave 1 landed the shared stack baseline and the parallel text insert prerequisite. Wave 2 turns the two import-critical fast paths into reviewable increments: direct attachment materialization and the deepwell render finalizer. The target is to produce merged, independently testable PRs before either path grows past the repository change-size guidance.

## Dependencies

- Base branch: `origin/develop` after PRs #285-#289.
- Design source: `docs/superpowers/specs/2026-07-08-runtime-50x-design.md`.
- No task may mutate the running `parityint20260630` stack during static implementation or unit tests.
- Browser-visible parity evidence remains a later gate; unit tests and static checks only prove the operator fast path shape.

## Task 5a: direct attachment planning mode

Branch: `perf/attachment-direct-plan`

Owned files: `install/local/wikidot-verification/scripts/apply-corpus-import-manifest.mjs`, new `install/local/wikidot-verification/src/corpus-attachment-direct.mjs`, and focused Node tests.

Interface: add `--attachment-create-mode rpc|direct`, default `rpc`, separate from the existing page `--create-mode rpc|db`.

Behavior for this slice: direct mode is dry-run only. It scans selected manifest rows, validates attachment bytes against corpus sha256 metadata, computes SHA-512 hex keys for the future content-addressed S3 upload, deduplicates by SHA-512, and adds `attachment_direct_plan` to the dry-run JSON with requested count, unique blob count, duplicate blob count, total bytes, and unique bytes. Non-dry-run direct mode fails closed with a clear not-implemented message so it cannot silently fall back to RPC.

Validation: `node --test install/local/wikidot-verification/tests/corpus-attachment-direct.test.mjs` and `node --test install/local/wikidot-verification/tests/corpus-import-manifest.test.mjs`.

## Task 5b: direct blob uploader

Branch: `perf/attachment-direct-upload`

Owned files: the direct attachment helper plus any small local S3 client support needed by the operator script.

Behavior: extend direct mode with a blob phase that uploads unique non-empty blobs to MinIO/S3 under the SHA-512 key using bounded concurrency, HEAD-first idempotency, size verification, and fail-closed per-blob results. The phase must not insert `file` or `file_revision` rows yet.

Validation: unit tests with an injected fake object-store adapter, plus a static dry-run test that proves the default RPC path remains unchanged. No live MinIO mutation in CI or routine validation.

## Task 5c: direct attachment staging SQL

Branch: `perf/attachment-direct-staging`

Owned files: the direct attachment helper, `apply-corpus-import-manifest.mjs`, and focused SQL-builder tests.

Behavior: add a staging-table SQL builder that joins planned attachments to existing imported pages and validates active filename uniqueness, existing matching rows, S3 hash mismatch failures, blob blacklist failures, and first-revision shape. The SQL builder should be testable without a live database by snapshotting the generated SQL fragments and by parsing synthetic result rows.

Validation: focused Node tests for match/skip/fail parsing and generated SQL guard clauses.

## Task 5d: direct attachment commit mode

Branch: `perf/attachment-direct-commit`

Owned files: the direct attachment helper and apply script.

Behavior: wire the upload phase and staging SQL into non-dry-run `--attachment-create-mode direct`, requiring `--db-url` and explicit S3 configuration. Output JSONL rows must keep the existing result shape: `attachments_requested`, `attachments_uploaded`, `attachments_skipped_existing`, and per-row failure fields. The path deliberately does not outdate pages because the render finalizer owns page render freshness.

Validation: fake-adapter tests, SQL parsing tests, and a small local-lab pilot only after code review; pilot evidence must compare RPC and direct rows for a selected attachment subset.

## Task 6a: render finalizer claim and health skeleton

Branch: `perf/render-finalizer-claim-loop`

Owned files: new deepwell render-finalizer module plus the existing runtime-action entrypoint.

Behavior: add a runtime action for the finalizer that can list and claim `render_pending` rows from `wikidot_corpus_import_item` using the existing lease columns, then immediately release them without rendering when `dry-run` is set. This PR owns durable queue mechanics only: import run selection, lease timeout, batch size, attempt limits, and JSON summary output.

Validation: Rust unit tests for SQL/state parsing where practical, `cargo fmt --manifest-path deepwell/Cargo.toml --check`, and `RUSTFLAGS='-D warnings' cargo clippy --manifest-path deepwell/Cargo.toml --tests --no-deps`.

## Task 6b: render finalizer pass 1

Branch: `perf/render-finalizer-pass1`

Owned files: the render-finalizer module and a narrow page revision render hook if needed.

Behavior: claimed rows are rendered through the real `PageRevisionService::rerender` path with bounded concurrency, pass-1 outdating suppression, per-row `rendered` or `render_failed` state, and resumable attempts. This task must avoid growing `deepwell/src/services/render/service.rs`.

Validation: focused Rust tests where possible, clippy/fmt, and a small local-lab pilot after review that reports rows/sec and render_failed taxonomy.

## Task 7: dependency-sensitive pass 2 and reporting

Branch: `perf/render-finalizer-pass2-health`

Owned files: the render-finalizer module and verification tooling only.

Behavior: identify Backlinks/nav/template-sensitive pages after pass 1, rerender that tail after the link graph exists, and emit a render-health summary compatible with the existing local Wikidot verification evidence directories.

Validation: focused tests for selector/query of pass-2 candidates, render-health sweep against the existing pilot verdicts, and browser parity evidence before merge if this changes user-visible output.

## Merge policy

Each task is a separate PR. Tasks 5a, 5b, 5c, and 6a can proceed independently once base `develop` is current. Task 5d depends on 5b and 5c. Task 6b depends on 6a and the already-merged `TextService::create` conflict fix. Task 7 depends on 6b.

## Risk controls

- Direct attachment code must never hide row mismatches; every mismatch is a per-row failure with JSONL evidence.
- Direct mode must not silently fall back to RPC.
- Render finalizer code must use product render services, not a Node-side FTML shortcut.
- Outdating suppression is only for finalizer pass 1 and must not affect normal edits or RPC rerenders.
- Every branch must be rebased after earlier wave-2 merges because `develop` requires strict up-to-date checks.
