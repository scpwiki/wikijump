# AGENTS.md: Wikijump

## Document map

- Start with `README.md` for project purpose.
- Use `docs/dom-compatibility.md` for DOM compatibility expectations.
- Use `docs/compatibility-ids.md` for imported Wikidot id and URL compatibility.
- Use `deepwell/README.md` for Deepwell's trusted-internal-API boundary.
- Use `docs/ftml-boundary.md` for the FTML/Wikijump responsibility boundary, pin-bump canary rule, and syntax-shim deviation process.
- Use `install/local/wikidot-verification/` for browser parity capture and validator tooling.
- Use `install/local/wikidot-verification/docs/sandbox-oracle-design.md` for the driftless sandbox-oracle design: fixture condition matrix, comparison layers, and the live-Wikidot mutation-allowlist sign-off requirement.

## Product language

Wikijump is a Wikidot-compatible local runtime. For imported Wikidot content, source-of-truth is live Wikidot evidence or corpus data with provenance. Local Wikijump output is never its own oracle.

Browser-visible behavior matters: visible text, meaningful DOM structure, links, modules, includes, files, metadata, permissions, actor state, and network/resource behavior can all be product surface. Do not hide meaningful differences through CSS, broad normalization, source surgery, or validator shortcuts. If a difference is intentionally accepted, record the policy reason and evidence.

For rendered Wikidot content, this fork's priority is faithful emulation over modernization. Upstream tends to replace Wikidot's legacy quirks with cleaner modern equivalents; here those quirks are product surface, because imported pages and their author CSS and scripts depend on Wikidot's actual DOM shape and interaction. When a faithful rendering and a modern one both pass aggregate checks, prefer the faithful one; matching tag counts or a normalized comparator is not the same as operating the page or running its original CSS and scripts. This does not extend to reproducing security-relevant Wikidot behavior; escaping and sanitization boundaries hold regardless of what Wikidot does.

## Architecture boundaries

- FTML owns syntax parsing/rendering primitives. Wikijump owns runtime behavior that depends on site/page/query/import state. The frozen boundary contract is `docs/ftml-boundary.md`.
- For `ListPages` and `CountPages`, FTML should preserve delayed module structure; Wikijump owns selector parsing, query semantics, URL arguments, pagination, variable substitution, and runtime rendering.
- The legacy (Wikidot) and new (Wikijump) layouts are the fidelity-versus-modernization axis, not interchangeable styling; see `docs/dom-compatibility.md`. The legacy layout carries literal Wikidot DOM shape and interaction. Keep the two as distinct paths rather than collapsing one into the other, and land Wikidot-faithful DOM for syntax constructs as an FTML `Layout::Wikidot` branch (`docs/ftml-boundary.md`), not as new post-render rewriting in Deepwell's frozen compatibility shims.
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
- One worktree per active delivery thread, reused across that thread's tasks (fetch, branch, or reset inside it), not one per task. Create a second worktree only for implementation work that is genuinely concurrent with the first, not to keep something around for later.
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

## Merging is not delivering

A merged PR changes `develop`. It does not change the standing runtime, which serves whatever SHA its containers were built from. Reporting a browser-visible change as done because CI passed and the PR merged asserts something unverified: the URL still serves the old build.

After merging a change that alters anything a browser can observe, run the Tier 1 refresh in `install/standing/README.md`, then re-fetch the affected URL and compare it against the live Wikidot page. Quote the fetched result. Until that comparison exists, the accurate report is that the change is merged and not yet observable, never that the defect is fixed.

Deploying is not configuring either. A feature whose behavior depends on site or page state stays invisible until that state exists, so name the missing state explicitly rather than implying the surface is complete.
