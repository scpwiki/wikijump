[<< Return to the README](../README.md)

# Codex Cloud environment

## Purpose

Wikijump needs more than the Universal image's automatic setup for representative development and security work. The reviewed environment installs the native, Node.js, Rust, Python, database, cache, and object store dependencies used across the repository, then preloads dependency stores so the agent phase can run with internet access disabled.

Codex executes scripts stored in the environment settings rather than discovering repository files automatically. The files in `scripts/` are the canonical, reviewable copies; paste their complete contents into the settings and keep both pasted copies synchronized with the same script revision.

## Environment settings

| Setting | Value |
|---|---|
| Container image | Universal |
| Repository | `Rokurolize/wikijump` |
| Node.js package version | 24 |
| Rust package version | 1.95.0 |
| Setup script | Paste the complete contents of `scripts/codex-cloud-setup.sh` |
| Maintenance script | Paste the complete contents of `scripts/codex-cloud-maintenance.sh` |
| Agent internet access | Off |
| Secrets | None |

Configure these ordinary environment variables so they remain available during setup, maintenance, and the agent phase.

```text
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/wikijump_codex
REDIS_URL=redis://127.0.0.1:6379
S3_FILES_BUCKET=deepwell-files
S3_TEXT_BLOCKS_BUCKET=deepwell-text-blocks
S3_REGION_NAME=test
S3_PATH_STYLE=true
S3_CUSTOM_ENDPOINT=http://127.0.0.1:9000
S3_ACCESS_KEY_ID=minio
S3_SECRET_ACCESS_KEY=minio-codex-test
NODE_OPTIONS=--max-old-space-size=8192
RUSTFLAGS=-D warnings
```

These are disposable local test values, not secrets. Do not configure production credentials. Cloud secrets are removed before the agent phase and therefore cannot support integration tests that run later.

## Setup behavior

Setup runs on the trusted default branch. It installs the native build toolchain, `libmagic`, PyYAML, Docker and Compose command support, PostgreSQL, Redis, MinIO, Node.js 24, pnpm 11.12.0, Rust 1.95.0, `sqlx-cli` 0.8.6, `cargo-machete` 0.9.1, ShellCheck, and actionlint 1.7.12. Fixed standalone downloads are checked against reviewed SHA-256 values.

The script fetches the three pnpm lockfiles without lifecycle scripts, fetches all Rust workspaces with locked manifests, and installs the importer's Python requirements. It creates `wikijump-cloud-services`, which starts and health-checks PostgreSQL, Redis, and MinIO, then resets their disposable state.

Setup, maintenance, and agent execution use separate Bash sessions. The Node activation helper records the canonical NVM binary path in shell startup files so each later session selects Node.js 24. It removes the obsolete `/opt/wikijump/node24` symlink used by earlier script revisions because a cached rerun could turn that link into a self-reference.

## Maintenance behavior

On a cached task, Codex checks out the task branch before running the pasted maintenance script. Maintenance removes relative and checkout-owned entries from `PATH`, activates the trusted Node and Rust toolchains, and changes to `/` before dependency fetching.

The pnpm fetches use `--ignore-scripts` and `--ignore-pnpmfile`; task-controlled lifecycle scripts and `.pnpmfile` hooks therefore do not execute while maintenance has network access. Cargo fetches use absolute manifests, disable Git CLI fetching, and ignore system and global Git configuration. Maintenance does not run migrations, seeders, builds, tests, or repository programs.

Maintenance invokes the setup-created service helper and resets PostgreSQL, Redis, and MinIO for every task, preventing cached state from leaking across branches.

## Agent phase

Run branch-controlled installation and migrations only after maintenance, when agent internet access is Off.

```bash
pnpm --dir framerail install --offline --frozen-lockfile
pnpm --dir install/local/wikidot-verification install --offline --frozen-lockfile
pnpm --dir locales/typed install --offline --frozen-lockfile
sqlx migrate run --source deepwell/migrations
```

Run seeders only for tests that require seeded application data. Full Docker image builds require a platform-provided daemon or socket and are not guaranteed in Codex Cloud; GitHub Actions remains the proof for image builds, while `docker compose config` can validate Compose structure locally.

## Acceptance and updates

Every script run begins with a revision banner. Setup and maintenance revisions in the Cloud log must match each other and the repository copies. An older or mismatched banner means the settings contain stale content.

Validate changes locally before updating the environment settings.

```bash
bash -n scripts/codex-cloud-setup.sh scripts/codex-cloud-maintenance.sh
shellcheck scripts/codex-cloud-setup.sh scripts/codex-cloud-maintenance.sh
python3 -m unittest scripts/codex_cloud_test.py
```

After a material script, package, runtime, or native dependency change, paste both current scripts, save the environment, reset its cache, and run one representative offline test from each affected surface. Reset the cache when maintenance reports missing system tools, a missing service helper, or a toolchain mismatch.

## Security boundary

Keep agent internet access Off for normal development and security scans. Setup and maintenance retain network access for dependency preparation, so they must remain pasted reviewed programs rather than commands that execute their task-branch copies. If browser parity work needs live domains, use a separate environment limited to the required read-only domains and `GET`, `HEAD`, and `OPTIONS`.

OpenAI documents the setup and maintenance order, separate Bash sessions, environment variable lifetime, secret removal, and container caching in [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environment). OpenAI documents the default-off policy and least-privilege controls in [Agent internet access](https://learn.chatgpt.com/docs/cloud/internet-access). pnpm documents that `fetch` uses lockfile data and that `.pnpmfile` files can execute resolution and fetching hooks in [pnpm fetch](https://pnpm.io/cli/fetch) and [.pnpmfile.mjs](https://pnpm.io/pnpmfile).
