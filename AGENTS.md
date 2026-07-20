# AGENTS.md: Wikijump

## Document map

- Start with `README.md` for project purpose.
- Use `docs/dom-compatibility.md` for DOM compatibility expectations.
- Use `docs/compatibility-ids.md` for imported Wikidot id and URL compatibility.
- Use `deepwell/README.md` for Deepwell's trusted-internal-API boundary.
- Use `docs/ftml-boundary.md` for the FTML/Wikijump responsibility boundary, pin-bump canary rule, and syntax-shim deviation process.
- Use `install/local/wikidot-verification/` for browser parity capture and validator tooling.

## Product language

Wikijump is a Wikidot-compatible local runtime. For imported Wikidot content, source-of-truth is live Wikidot evidence or corpus data with provenance. Local Wikijump output is never its own oracle.

Browser-visible behavior matters: visible text, meaningful DOM structure, links, modules, includes, files, metadata, permissions, actor state, and network/resource behavior can all be product surface. Do not hide meaningful differences through CSS, broad normalization, source surgery, or validator shortcuts. If a difference is intentionally accepted, record the policy reason and evidence.

## Architecture boundaries

- FTML owns syntax parsing/rendering primitives. Wikijump owns runtime behavior that depends on site/page/query/import state. The frozen boundary contract is `docs/ftml-boundary.md`.
- For `ListPages` and `CountPages`, FTML should preserve delayed module structure; Wikijump owns selector parsing, query semantics, URL arguments, pagination, variable substitution, and runtime rendering.
- Unsupported or unverified query/module shapes must fail closed, remain literal, or use an evidenced fallback. Do not silently drop selectors or widen queries.
- Imported Wikidot uploaded files are data. Do not add real article/page uploaded assets to repository seed data as a parity fix; use corpus file capture plus import into local Wikijump file state.
- Real EN/JP Wikidot sites are read-only unless the user explicitly authorizes a run-owned sandbox mutation.
- Imported corpus data (runtime database/files volumes) is expensive to recreate. Never run `docker compose down -v` against a runtime stack that holds it; treat volume deletion as data loss requiring explicit user authorization.

## Implementation rules

- Land one candidate at a time: this repository has no native merge queue, so confirm no other PR is in the landing lane before pushing a candidate to merge.
- Size PRs by reviewability and risk, not by a line quota. Do not split a coherent change merely to shrink the diff, and do not let foundation or hardening PRs multiply before the capability they serve has produced its first end-to-end result.
- High-touch render code needs extra care: before touching `deepwell/src/services/render/service.rs`, search for existing helpers and nearby tests; for broad renderer changes, add focused tests plus regression canary evidence before merge.
- Target Rust modules under roughly 500 LoC excluding tests; past roughly 800 LoC, put new functionality in a new module and move the related tests and docs with it.
- Use isolated worktrees for implementation in a shared environment. A root checkout is a read-only reference; do not leave edits, artifacts, commits, or dirty state there unless the user assigned it.
- Create new worktrees under `~/wjlab/worktrees/wikijump/<task-slug>`. Do not scatter worktrees across other locations; this keeps host-wide cleanup and auditing tractable. Existing worktrees elsewhere may stay until their task closes.
- Keep repository code, private data, local DB state, and generated evidence separate. Never commit credentials, cookies, browser profiles, auth JSON, raw private dumps, or local DB dumps.

## Resource lifecycle

- Label every run-owned docker resource (container, volume, image, network) and every worktree at creation with its owning lane and an expiry; name data volumes explicitly, never anonymously.
- Every receipt that pauses, closes, or supersedes a lane must include a resource-disposition section covering each container, volume, image, worktree, and target dir the lane created, each marked keep-until a date or delete-now. A receipt without this section is incomplete.
- Superseded-candidate teardown happens in the same closure step that declares supersession, not in a later cleanup pass.
- If a resource must outlive its lane, its closing receipt records the new owner and expiry; nothing keeps running detached after its lane closes.

## Validation expectations

Run the narrowest meaningful validation for the touched surface, then broaden before PR/merge when behavior is user-visible or cross-cutting.

```bash
cargo fmt --manifest-path deepwell/Cargo.toml --check
cargo test --manifest-path deepwell/Cargo.toml <focused-test-or-module>
RUSTFLAGS='-D warnings' cargo clippy --manifest-path deepwell/Cargo.toml --tests --no-deps
pnpm --dir framerail build
pnpm --dir framerail lint
node --test install/local/wikidot-verification/tests/<focused-test>.mjs
```

For browser-visible parity claims, also produce fresh source/local browser evidence with the local Wikidot verification tooling; unit tests and DB rows are supporting evidence only.

Before merging, verify current branch protection and required checks from GitHub. Do not bypass required checks, force/admin merge, or push to upstream `scpwiki/*` repositories.
