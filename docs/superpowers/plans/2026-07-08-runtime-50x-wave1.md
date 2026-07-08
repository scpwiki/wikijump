# Runtime 50x Wave 1 Implementation Plan (spec slices 1-4)

> For agentic workers: execution of this plan is delegated to Codex workers
> (one per task, dedicated worktree each) per the codex-first process, with
> Claude-side review after every task. Steps use checkbox (`- [ ]`) syntax for
> tracking. Tasks are independent and run in parallel.

Goal: land the four small independent slices of the runtime-50x spec — sqlx
logging switch, local release/built toggles, lab-fenced Postgres tuning, and
the text-insert race fix — so the stack baseline exists for every later
measurement.

Architecture: see `docs/superpowers/specs/2026-07-08-runtime-50x-design.md`
(workstream 1 and the finalizer prerequisite from workstream 3). Each task is
one PR.

Tech stack: Rust (deepwell, sea-orm), docker compose, shell, SvelteKit
(framerail).

## Global constraints

- Default behavior unchanged: every new knob defaults to today's behavior.
- No changes to the running `parityint20260630` stack; do not restart or
  recreate any live container. Static verification only.
- Validation floor per task: `cargo fmt --manifest-path deepwell/Cargo.toml
  --check`, `RUSTFLAGS='-D warnings' cargo clippy --manifest-path
  deepwell/Cargo.toml --tests --no-deps` (Rust tasks), plus the focused test
  commands listed in the task.
- Change size well under 500 lines per task.
- Commit messages: imperative, no attribution trailers other than the
  session link added at PR time by the orchestrator.

---

### Task 1: sqlx logging config switch

Branch: `perf/sqlx-logging-config`

Files:
- Modify: `deepwell/src/database/mod.rs:29-44`
- Modify: `deepwell/src/config/file.rs` (struct `Database` at :87, its
  destructure ~:270, conversion ~:413, default ~:549, test ~:682)
- Modify: `deepwell/src/config/object.rs` (add flattened field)
- Modify: `deepwell/src/api.rs:99` (call site)
- Modify: `install/local/deepwell/config.toml` (`[database]` section, :9-11)

Interfaces:
- Produces: TOML key `sqlx-logging` (kebab-case, matching `run-seeder`) in
  `[database]`, default `true`; config object field `sqlx_logging: bool`;
  new signature `database::connect(database_uri, sqlx_logging: bool)`.

- [ ] Step 1: add `sqlx_logging: bool` to the file-config `Database` struct
  with a serde default of `true` (follow the pattern used by existing
  defaulted fields in `file.rs`; if none exists, use
  `#[serde(default = "default_true")]` with a module-local
  `fn default_true() -> bool { true }`).
- [ ] Step 2: plumb the field through the destructure and into the flattened
  config object next to `run_seeder`, updating the built-in default (~:549)
  to `true`.
- [ ] Step 3: change `database::connect` to accept `sqlx_logging: bool` and
  pass it to `.sqlx_logging(...)`:

```rust
pub async fn connect<S: Into<String>>(
    database_uri: S,
    sqlx_logging: bool,
) -> Result<DatabaseConnection> {
    ...
    options
        .min_connections(4)
        .max_connections(100)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(10))
        .sqlx_logging(sqlx_logging);
    ...
}
```

- [ ] Step 4: update the single call site `deepwell/src/api.rs:99` to pass
  `config.sqlx_logging` (confirm the config object is in scope there; it is
  where `database_url` comes from).
- [ ] Step 5: add `sqlx-logging = false` to `install/local/deepwell/
  config.toml` under `[database]` with a one-line comment that this is a
  local-lab performance setting.
- [ ] Step 6: extend the existing config parse test (file.rs ~:682) to assert
  the default is `true` when the key is absent and `false` when set false.
- [ ] Step 7: run `cargo test --manifest-path deepwell/Cargo.toml config`,
  expect pass; run the fmt/clippy validation floor, expect clean.
- [ ] Step 8: commit: `feat(deepwell): make sqlx statement logging configurable`

### Task 2: local compose release/built toggles

Branch: `perf/local-release-toggles`

Files:
- Modify: `install/local/deepwell/deepwell-start:12-18`
- Modify: `install/local/framerail/Dockerfile:20-21`
- Create: `install/local/framerail/framerail-start`
- Modify: `install/local/docker-compose.yaml` (pass the two env vars through
  to the deepwell and framerail services with default values)

Interfaces:
- Produces: env `DEEPWELL_BUILD_PROFILE` = `debug` (default) | `release`;
  env `FRAMERAIL_MODE` = `dev` (default) | `built`.

- [ ] Step 1: in `deepwell-start`, branch on `DEEPWELL_BUILD_PROFILE`:

```sh
PROFILE_FLAG=""
if [ "${DEEPWELL_BUILD_PROFILE:-debug}" = "release" ]; then
    PROFILE_FLAG="--release"
fi
exec /usr/bin/env RUST_BACKTRACE=1 \
    /usr/local/cargo/bin/cargo watch \
        --why \
        -w /src/deepwell \
        -w /opt/locales \
        -w /etc/deepwell.toml \
        -x "run $PROFILE_FLAG -- /etc/deepwell.toml"
```

- [ ] Step 2: create `framerail-start` (sh): default mode runs `pnpm dev`
  exactly as today; `FRAMERAIL_MODE=built` runs `pnpm build` then starts the
  SvelteKit adapter output. Inspect `framerail/svelte.config.*` to confirm
  the adapter (expected adapter-node; start with `node build`) and match the
  port/env the compose file expects (3393, `FRAMERAIL_ENV=local`). If the
  adapter is not adapter-node, stop and report instead of improvising.
- [ ] Step 3: switch the framerail Dockerfile CMD to the start script (COPY
  it like `health-check.sh` is copied) and keep default behavior identical.
- [ ] Step 4: wire both env vars through `install/local/docker-compose.yaml`
  with defaults (`${DEEPWELL_BUILD_PROFILE:-debug}`, `${FRAMERAIL_MODE:-dev}`).
- [ ] Step 5: verify statically: `sh -n` both scripts; `docker compose -f
  install/local/docker-compose.yaml config` renders (expect success, no
  value change when env unset — diff the rendered config against a pre-change
  render to prove default equivalence). Do not start containers.
- [ ] Step 6: commit: `feat(local): opt-in release deepwell and built framerail`

### Task 3: lab-fenced Postgres performance override

Branch: `perf/pg-lab-override`

Files:
- Create: `install/local/docker-compose.postgres-perf.yaml`
- Modify: `install/local/README.md` if present (one paragraph documenting the
  override; if absent, document at the top of the override file only)

Interfaces:
- Produces: an optional compose override enabling
  `synchronous_commit=off`, `fsync=off`, `full_page_writes=off` on the local
  database service.

- [ ] Step 1: confirm the database service name and image in
  `install/local/docker-compose.yaml`, then create the override:

```yaml
# Lab-only Postgres durability relaxations for bulk import performance.
# FENCED TO THE DISPOSABLE LOCAL LAB: these settings can corrupt data on
# crash. The local compose database uses an anonymous volume that is reset
# on container recreation, so durability loss here is acceptable by
# construction. Never use outside install/local.
#
# Usage:
#   docker compose -f docker-compose.yaml -f docker-compose.postgres-perf.yaml up -d
services:
  database:
    command: >-
      postgres
      -c synchronous_commit=off
      -c fsync=off
      -c full_page_writes=off
```

  (Adjust the service key to the actual name found in step 1; if the base
  service already defines `command`, extend that command instead.)
- [ ] Step 2: verify `docker compose -f install/local/docker-compose.yaml -f
  install/local/docker-compose.postgres-perf.yaml config` renders and shows
  the command on the database service. Do not start containers.
- [ ] Step 3: commit: `feat(local): optional lab-only postgres perf override`

### Task 4: TextService::create insert race fix

Branch: `perf/text-insert-conflict`

Files:
- Modify: `deepwell/src/services/text.rs:167-186`
- Test: alongside existing text service tests if any exist (search
  `deepwell/src` and `deepwell/tests` for text-service tests first; if the
  test harness requires a live DB and none exists for this service, state
  that in the report instead of inventing a mock layer).

Interfaces:
- Consumes/produces: `TextService::create(ctx, contents) -> Result<TextHash>`
  signature unchanged; behavior under concurrent identical inserts changes
  from unique-violation error to idempotent success.

- [ ] Step 1: keep the cheap `exists` fast path (it avoids shipping large
  contents to Postgres), and make the insert itself conflict-safe:

```rust
use sea_orm::sea_query::OnConflict;

if !exists {
    let model = text::ActiveModel {
        hash: Set(hash.to_vec()),
        contents: Set(contents),
    };

    Text::insert(model)
        .on_conflict(
            OnConflict::column(text::Column::Hash)
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(txn)
        .await
        .or_raise(make_error)?;
}
```

  Note: use `exec_without_returning`, not `exec` — with sea-orm, a
  do-nothing conflict on `exec` surfaces as `DbErr::RecordNotInserted`,
  which would turn the benign race back into an error. Verify against the
  sea-orm version in `deepwell/Cargo.toml` and adapt if its API differs.
- [ ] Step 2: check how other services in this repo already handle
  insert-on-conflict (grep `OnConflict` under `deepwell/src`) and match the
  house idiom if one exists.
- [ ] Step 3: run `cargo test --manifest-path deepwell/Cargo.toml text`,
  expect pass (or no matching tests); run the fmt/clippy floor, expect clean.
- [ ] Step 4: commit: `fix(deepwell): make text insert race-safe under parallel render`

---

## Out of scope for wave 1

Slices 5-9 of the spec (attachment direct materializer, render finalizer
core and pass 2, consolidated article-view RPC, response cache) get their own
plan documents once wave 1 lands and the release-stack measurements exist.
The wave-1 evidence run (page-view latency sample + shell-registration rerun,
before/after) is orchestrator work, not a Codex task, because it touches the
live lab stack.
