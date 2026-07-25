# What each workflow is for

Every workflow here costs wall-clock time on a pull request, so each one needs
a reason to exist and a trigger narrow enough that it only runs when it can
actually say something. This file records both. `scripts/preflight.sh` runs the
local half of the same checks, so most failures can be found before pushing.

## Pull-request validation

`ci-gate.yaml` is the required gate. It classifies the changed paths with
`.github/scripts/classify-changes.mjs` and then runs only the affected areas,
which is why an unrelated change does not pay for the whole matrix. Its jobs:

- Classify changes: derives the `deepwell`, `wws`, `framerail`, `locales`, and
  `workflow` outputs every other job keys off.
- Workflow policy: `actionlint` plus `.github/tests/`, which assert the CI
  structure itself, including that third-party actions are pinned to full
  commit SHAs and that the Framerail unit and browser suites stay separate.
- Deepwell lint and unit tests: the draft path, which skips the database.
- Deepwell lint, unit, and integration tests: the candidate path, which brings
  up Postgres, Valkey, and MinIO and runs migrations and the seeder. This is
  the slowest job on a normal PR at roughly ten minutes.
- WWS, Framerail, Locales: the per-area equivalents.
- CI / gate: the single required check that aggregates the rest, so branch
  protection has one status to require rather than a list that changes.

`full-ci.yaml` is opt-in through the `full-ci` label and carries what is too
slow for every push: coverage for Deepwell and WWS, the coverage exports, and
the Playwright browser suite. Requiring it on every PR would make routine work
unbearable; requiring it on nothing would let coverage and browser regressions
land. The label is the compromise, which is why merge candidates carry it.

`source-size.yaml` enforces `scripts/source-size-baseline.txt`. It runs on
every pull request because it is repository-wide and cheap, and because it is
the check most likely to fail on a *merge result* rather than on either branch:
two PRs that each add to one file can both pass alone and fail together. The
`pre-commit` hook runs the same script so a file crossing its ceiling is caught
as it is written.

`codeql.yaml` runs the security analyses on pushes to `develop` and `prod`, on
pull requests to `develop`, and on a schedule. The schedule matters
independently of the PR runs: new queries find old code.

## Post-merge and deployment

`docker-build-*.yaml` build the container images per service and environment,
all delegating to `docker-build-template.yaml` so the build logic exists once.
`docker-push-minio.yaml` publishes the MinIO image, and is path-filtered to
`install/local/minio/*` because nothing else can change it.

`komodo-deploy.dev.yaml` and `komodo-deploy.prod.yaml` deploy on pushes to
`develop` and `prod` respectively.

## Narrowly scoped

`wikidot-verification.yaml` runs the verification tooling's own tests, filtered
to `install/local/wikidot-verification/**` and `install/standing/**`. It does
not attempt live Wikidot capture: that needs credentials and mutates a sandbox,
so it stays a local operation with human authorization.

`codex-cloud.yaml` validates the Codex cloud environment scripts, path-filtered
to those scripts and their documentation.

## Keeping triggers honest

A guard that reads a file must run when that file changes. The workflow policy
tests assert about `framerail/package.json` and `framerail/playwright.config.ts`
while living under `.github/`, so `classify-changes.mjs` carries an explicit
`WORKFLOW_POLICY_SUBJECTS` list to select the `workflow` group for them. Without
it the guard was unable to fire on the change that broke it, and the violation
surfaced on an unrelated pull request days later. If you add an assertion about
a new file outside `.github/`, add it to that list.
