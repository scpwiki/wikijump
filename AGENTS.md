# AGENTS.md: Wikijump

## Document map

- Start with `README.md` for project purpose.
- Use `docs/dom-compatibility.md` for DOM compatibility expectations.
- Use `docs/compatibility-ids.md` for imported Wikidot id and URL compatibility.
- Use `deepwell/README.md` for Deepwell's trusted-internal-API boundary.
- Use `install/local/wikidot-verification/` for browser parity capture and validator tooling.

## Product language

Wikijump is a Wikidot-compatible local runtime. For imported Wikidot content, source-of-truth is live Wikidot evidence or corpus data with provenance. Local Wikijump output is never its own oracle.

Browser-visible behavior matters: visible text, meaningful DOM structure, links, modules, includes, files, metadata, permissions, actor state, and network/resource behavior can all be product surface.

Do not hide meaningful differences through CSS, broad normalization, source surgery, or validator shortcuts. If a difference is intentionally accepted, record the policy reason and evidence.

## Architecture boundaries

- FTML owns syntax parsing/rendering primitives. Wikijump owns runtime behavior that depends on site/page/query/import state.
- For `ListPages` and `CountPages`, FTML should preserve delayed module structure; Wikijump should own selector parsing, query semantics, URL arguments, pagination, variable substitution, and runtime rendering.
- Unsupported or unverified query/module shapes must fail closed, remain literal, or use an evidenced fallback. Do not silently drop selectors or widen queries.
- Imported Wikidot uploaded files are data. Do not add real article/page uploaded assets to repository seed data as a parity fix. Use corpus file capture plus import/materialization into local Wikijump file state.
- Real EN/JP Wikidot sites are read-only unless the user explicitly authorizes a run-owned sandbox mutation.

## Implementation rules

Prefer small PR-sized slices with a clear root cause. Separate materialization, source freshness, renderer syntax, page chrome, file/resource availability, ListPages behavior, validator changes, and cleanup.

High-touch render code needs extra care. If touching `deepwell/src/services/render/service.rs`, first search for existing helpers and nearby tests. Avoid unbounded growth in central render code; extract or group logic when it clarifies ownership. For broad renderer changes, add focused tests plus regression canary evidence before merge.

For page/file/import work, keep repository code, private data, local DB state, and generated evidence separate. Do not commit credentials, cookies, browser profiles, auth JSON, raw private dumps, or local DB dumps.

Use isolated worktrees for implementation when operating in a shared local environment. A root checkout may be used as a fetch/read-only reference, but do not leave implementation edits, generated artifacts, commits, or dirty state there unless the user explicitly assigned that checkout as the worktree.

Avoid large modules:

- Prefer adding new modules instead of growing existing ones.
- Target Rust modules under 500 LoC, excluding tests.
- If a file exceeds roughly 800 LoC, add new functionality in a new module instead of extending the existing file unless there is a strong documented reason not to.
- When extracting code from a large module, move the related tests and module/type docs toward the new implementation so the invariants stay close to the code that owns them.

### Change size guidance (800 lines)

Unless the change is mechanical the total number of changed lines should not exceed 800 lines.
For complex logic changes the size should be under 500 lines.

If the change is larger, explore whether it can be split into reviewable stages and identify the smallest coherent stage to land first.
Base the staging suggestion on the actual diff, dependencies, and affected call sites.

## Validation expectations

Run the narrowest meaningful validation for the touched surface, then broaden before PR/merge when behavior is user-visible or cross-cutting.

Common commands to choose from:

```bash
cargo fmt --manifest-path deepwell/Cargo.toml --check
cargo test --manifest-path deepwell/Cargo.toml <focused-test-or-module>
RUSTFLAGS='-D warnings' cargo clippy --manifest-path deepwell/Cargo.toml --tests --no-deps
pnpm --dir framerail build
pnpm --dir framerail lint
node --test install/local/wikidot-verification/tests/<focused-test>.mjs
```

For browser-visible parity claims, also produce fresh source/local browser evidence with the local Wikidot verification tooling. Unit tests and DB rows are supporting evidence only.

Before merging, verify current branch protection and required checks from GitHub. Do not bypass required checks, force/admin merge, or push to upstream `scpwiki/*` repositories.
