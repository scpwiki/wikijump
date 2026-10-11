# Test quality audit: coverage, mutation, and ownership

This document records how repository test quality is measured so results are
reproducible and so a coverage number is never mistaken for an ownership
judgement. It supports issue #1990 and the Deepwell owner-boundary work in
#1977.

## Opt-in audit commands and reviewed ledger

`docs/development/test-quality-audit.json` binds the production denominator to
current source, test, frozen input, manifest, and lockfile bytes. The inventory
starts from checkout files, not a coverage report. It resolves relative module
imports, Rust child test modules, JavaScript wrapper suites, Python tests,
Deepwell binaries, and the separate relation proc-macro sources. Imported suites
retain their named assertions but are not rerun as independent Node entrypoints.
Discovery is navigation; an import or source-shaped registry witness alone does
not establish behavioral ownership.

```sh
node scripts/run-test-quality-audit.mjs inventory --output-dir /tmp/audit-inventory
node scripts/run-test-quality-audit.mjs coverage --output-dir /tmp/audit-coverage
node scripts/run-test-quality-audit.mjs coverage --owner resource-scanner --output-dir /tmp/audit-resource-coverage
node scripts/run-test-quality-audit.mjs mutate --owner generated-gate-close-unit --output-dir /tmp/audit-gate-unit
node scripts/run-test-quality-audit.mjs mutate --owner generated-gate-close-listpages --output-dir /tmp/audit-gate-rendered
node scripts/run-test-quality-audit.mjs verify --output-dir /tmp/audit-verification
```

Every output directory must be new and outside the checkout. Commands remain
opt-in; no coverage or mutation jobs are added to CI. Measurements enter the
normal external-network guard and use existing locked dependencies. Prepare the
pinned FTML and wikidot.py checkouts using the existing `WIKIJUMP_FTML_CHECKOUT`
and `WIKIDOT_PY_CHECKOUT` contracts when their default sibling paths are absent.

### Pinned source snapshot and freshness

`snapshot` in the ledger pins every production source and frozen input hash to
one exact commit and tree. The CI gate (`scripts/verify-test-quality-evidence-identities.mjs`)
recomputes those hashes from the pinned commit's Git objects, so the snapshot is
checked for provenance without comparing it to the working tree. Ordinary
implementation edits never need a hand-entered hash. In a checkout whose source
differs from the snapshot, the gate lists the changed identities as stale and still
passes. Stale output is informational; it is not a test result and does not claim
behavioral coverage.

The gate fails for malformed or missing snapshot metadata, a pinned tree or hash
that disagrees with its commit, an unavailable pinned commit (GitHub Actions
fetches it by SHA with `--depth=1`; local runs must have the history), a missing
required input, or a production source outside the declared scope. Each failure
names the exact refresh command.

Regenerate the snapshot only when recording new audit evidence, for example
after a reviewed record or owner changes, the inventory gains or loses production
files, the frozen lock or toolchain inputs change, or before final closure:

```sh
node scripts/refresh-test-quality-snapshot.mjs --commit <audited-commit> --check
node scripts/refresh-test-quality-snapshot.mjs --commit <audited-commit>
```

The refresh regenerates identities from a temporary worktree at the named commit,
verifies them against that commit's Git objects, preserves existing review and
finding state, and refuses to drop a reviewed record or orphan an owner. Refreshing
a hash is provenance only. It is not test evidence, and mutation or coverage
proofs must be rerun against the new inventory.

The ledger currently keeps unaudited files pending. `verify` rejects unresolved
records and missing final acceptance. `verify --allow-incomplete` can check a
work-in-progress ledger without authorizing issue closure. Neither passing
baselines nor a high coverage percentage changes that rule. Historical issue
comments are provenance until their exact source, test, tool, and inventory
identities are reconciled.

Terminal records require named executable owners, a source-bound material-region
assessment, independent review, and supporting proof references. Proof admission
opens and hashes supporting artifacts. Mutation acceptance cross-checks the
frozen inventory, successful baseline, command, target, log identity, restoration,
and each individual outcome. Survivors, compilation failures, and demonstrated
mutation-induced nonprogress retain separate dispositions. Arbitrary evidence
strings cannot authorize acceptance. Final closure additionally requires current
portable, proc-macro, generated-contract, scanner, and final-preflight evidence;
a verified normal two-parent merge; and applicable merged-head standing proof.

The former 39-entry generated-gate recovery inventory is retained only in
the dated evidence package. The current develop implementation removed that
recovery helper, so those receipts do not bind the merged source tree and are
not admitted by the active ledger. Any future mutation proof must target a
current executable owner. Standing volumes and persistent local development
services are never part of the audit lifecycle.

## Measured coverage

### Deepwell (Rust)

The combined measurement retains unit, integration, and combined JSON reports
with line, function, and region denominators. It seeds a disposable stack using
the normal uninstrumented build, resolves the seed binary through Cargo metadata,
then retains unit profiles separately before collecting serial integration
profiles. Seeder-only profiles cannot raise application coverage. The
`deepwell-relation-impl-derive` package has its own report and test invocation;
it is not counted as an implicitly exercised dependency.

Rust branch instrumentation requires nightly. Stable line/function/region
coverage and targeted mutation are the accepted measurements; a missing branch
percentage is a documented instrumentation limitation, not a coverage claim.
Tool versions, selected features, exact commands, and source/test/input hashes
must accompany reviewed measurements.

### Node, Vite SSR, and frozen subprocess environments

Node's built-in instrumentation reports line, function, and V8 branch coverage
without adding a new production seam. Framerail keeps its normal SvelteKit sync
and Vite SSR owners. Some modules are visible in reports; absent SSR modules
need a demonstrated instrumentation limitation and independently reviewed action
or browser ownership. A blanket SSR exclusion is not an assessment.

Instrumentation of a child process can fail when Node tries to add
`NODE_V8_COVERAGE` to a deliberately frozen environment. Preserve that environment
contract. Retain the failed instrumented probe, a passing uninstrumented baseline,
and the exact affected owner; measure compatible owners separately. Never turn
an unrelated test failure into an instrumentation exclusion.

Node coverage runs the compatible test entrypoints in sequential batches of at
most 25 files. Each batch writes its own LCOV report; the audit merges line,
function, and branch hits by source identity so a file exercised by multiple
batches is not double-counted. Batching also prevents one subprocess's empty V8
profile from discarding otherwise valid package coverage. If a frozen child
environment fails under instrumentation, retain that probe and the complete
passing baseline, then retry the exact owner in smaller batches. Exclude an owner
only if the failure remains reproducible after batching; the summary lists only
owners actually excluded from its test entrypoints.

The Wikidot verification package has 13 explicitly named child-process owners
whose deliberately frozen Git environments reject Node 24's injected
`NODE_V8_COVERAGE` property. The full uninstrumented baseline still runs every
entrypoint. The repository coverage command instruments every compatible owner
and records those exact exclusions; `test-quality-coverage.mjs` fails if any
named path disappears from the current inventory. The reproduced failures and
exact-owner reruns remain in
`docs/development/test-quality-audit/wikidot-verification-node-coverage-20261009/`;
the current full-run results for Deepwell, Framerail, and Wikidot verification
are retained in
`docs/development/test-quality-audit/test-quality-coverage-20261009/`.

## Mutation testing

Use `cargo-mutants` against a task-owned disposable integration stack, never
against the whole repository. A broad sweep is expensive, disk-heavy, and
low-signal.

1. Start a disposable PostgreSQL/Valkey/MinIO stack and seed it, mirroring
   `install/local/wikidot-verification/scripts/run-deepwell-integration-validation.mjs`
   (its MinIO `/data` is tmpfs so ephemeral storage does not depend on the host
   disk).
2. Export that stack's connection environment.
3. Run cargo-mutants in place, scoped to one file and its owning test target.

```sh
cargo mutants --in-place --manifest-path deepwell/Cargo.toml \
  -f src/services/context.rs \
  -F 'should_commit_authentication_rejection' \
  --baseline run \
  -C --test -C rpc_boundary
```

`--file` is relative to the manifest directory (`deepwell/`). `--in-place`
avoids duplicating the target tree; do not combine it with `--jobs`.
`-C --test -C <target>` restricts each mutant run to the owning test binary so
mutants in integration-owned code are judged by integration tests.

Outcome meanings:

- `caught` — a test killed the mutant.
- `unviable` — the mutated source does not compile; not a test gap.
- `timeout` — the mutant made the test phase nonterminating, or at least far
  slower than the baseline. Inspect it: a mutant-induced infinite loop is a
  useful detection, not a missing test. Record it separately from `missed`.
- `missed` — a survivor. It is a real gap only if the mutation changes an
  observable contract. Otherwise record it as equivalent/non-actionable with
  the specific consumer that makes it unobservable.

### Timeouts

Let cargo-mutants derive the test timeout from the unmutated baseline
(`--baseline run`). Do not pass a fixed `--timeout` by default: a mutant that
creates an infinite loop is then detected after a baseline-relative bound
instead of after an arbitrary long wait. If an override is genuinely needed,
prefer `--timeout-multiplier` over an absolute value, and record the measured
baseline test duration first. cargo-mutants does not impose a build timeout by
default; add `--build-timeout` only when a build hang has actually been
observed.

For the Deepwell lib suite (baseline test phase of a few seconds), a mutant
test should not receive a minutes-long budget without a specific reason.

If an in-place run must be interrupted, signal cargo-mutants itself (SIGINT),
then verify the mutated source was restored before doing anything else. Do not
kill the compiler/test children directly.

## Test layer

Match the test layer to the contract:

- `src/**` unit tests own small, pure, or implementation-local behavior:
  parsing primitives, selectors, normalization, encoding helpers, and
  deterministic transformations.
- `deepwell/tests/**` integration tests own behavior observable through the
  service boundary: JSON-RPC, authorization, persistence, transactions, cache
  invalidation, membership/forum behavior, and externally visible ListPages
  semantics.

A mutant is not adequately owned merely because a `--lib` run kills it when the
contract belongs at an integration boundary; conversely, do not build an
expensive service-backed test for a pure helper whose strongest independent
contract is a unit-level invariant. Identify the strongest existing owner
first, then run the test target that owns that behavior.

Do not add automatic retries to make a test green. If retries are used to
diagnose nondeterminism, record the test as flaky and treat it as a defect.
Prefer deterministic synchronization, isolated resources, bounded explicit
waits, and observable readiness conditions over sleeps.

Cargo builds each top-level `tests/*.rs` file as its own crate. Add a coherent
case to an existing behavioral integration target rather than creating a new
file for every small regression, unless measurement shows the layout is a real
bottleneck.

## Property-based testing

For a pure parser/scanner/selector/encoding component with a genuine general
invariant (round-trip, idempotence, no-panic, delimiter preservation,
normalization), `proptest` can add value over hand-picked cases because it
shrinks failures and persists seeds. Do not use it where the expected result
would be generated from the implementation under test, and do not replace
exact Wikijump compatibility fixtures with generated properties: observed
compatibility remains an independent oracle. Introduce the dependency only
after demonstrating the concrete gap it covers.

## Ownership vocabulary

When reviewing an uncovered region or a surviving mutant, classify it as
exactly one of:

- genuine missing regression
- integration-owned
- browser/runtime-owned
- startup/config-owned
- instrumentation blind spot
- unreachable/dead code (remove it rather than testing it)
- intentionally unsupported / fail-closed
- equivalent / non-actionable mutant

Only genuine missing regressions need new tests. Coverage percentage is
evidence, not the objective.

## Earlier mutation frontier

The mutation frontier recorded on 2026-10-08 is preserved at
docs/development/test-quality-audit-frontier-20261008.json, with its original
method notes in docs/development/test-quality-audit-frontier-20261008.md. Its
sharded replay driver remains available as
scripts/run-test-quality-frontier-audit.mjs; its CLI regression is retained
separately. Its 2026-10-08 identity snapshot is historical and is expected to
fail current-source drift checks after this merge; use the current audit runner
above for present-day verification. Those records are historical mutation evidence. The current
source/test ownership denominator and fail-closed proof admission are maintained
by the ledger and commands above.
