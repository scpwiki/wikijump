# Local Wikidot verification tools

The scripts in this directory import frozen Wikidot corpus data, inspect a local runtime, capture browser evidence, and reduce large runs into machine-readable verdicts. Expected behavior must come from the frozen corpus, reviewed compatibility policy, or sealed real-Wikidot evidence. Local Wikijump output is diagnostic evidence, not an oracle.

## Routine compatibility regression is offline

The completed compatibility campaign is an evidence-acquisition history, not the normal test loop. Routine development must consume repository-owned frozen observations and must not reacquire Wikidot, WDFiles, theme CDNs, fonts, or any other public compatibility dependency.

Run the hermetic contract/unit suite plus the checked-in syntax differential with:

```sh
pnpm --dir install/local/wikidot-verification offline
```

Run the product-owned WWS, Framerail, and Deepwell regression suites against
task-owned local services with external networking denied with:

```sh
pnpm --dir install/local/wikidot-verification offline:products
```

`offline:portable` runs both layers. This is the fresh-checkout compatibility
gate: it does not require `/home/roku/wjlab`, a campaign candidate, or any live
Wikidot endpoint. `fixtures/offline-compatibility/final-zero-surface-coverage.json`
freezes the exact 900-surface final-zero denominator and assigns every row to
repository-owned executable anchors. One hundred rows whose final-zero ledger
was backed only by candidate/campaign receipts or non-executable source anchors
are explicitly migrated to ordinary verification/product tests in that fixture;
the fixture validator rejects a host-specific WJLab path, a live Wikidot/WDFiles
dependency, a missing anchor file, or a lost denominator row.

Run the full-page SCP-9506 browser oracle against a local standing runtime with:

```sh
pnpm --dir install/local/wikidot-verification offline:browser
```

`offline:browser` uses `fixtures/offline-compatibility/scp-9506-final-zero-oracle.json`, the exact final-zero Chromium identity, a repository-owned external-response fixture, and the final-zero-accepted full-page Wikijump image. Public stylesheet/font/image responses are replayed from the fixture. Any missing public response fails closed instead of fetching it. The semantic comparison uses the frozen Wikidot browser observation; the visual comparison uses normalized ImageMagick RMSE against the Wikijump image that passed that same final-zero live comparison. The browser runs in a fresh user+network namespace with only loopback. A Unix-domain socket bridges that namespace's `127.0.0.1:443` exclusively to the host's existing standing `127.0.0.1:443`; Chromium has no external IP route at all. The harness locally fulfills only the exact `/local--favicon/favicon.gif` browser-internal request to prevent its standing redirect from escaping the sealed fixture model, and records that normalization in the verdict.

Run the portable gate plus the local-standing browser oracle with
`pnpm --dir install/local/wikidot-verification offline:all`. Live-reference,
preview-capture, authenticated probe, and sandbox-mutation commands below are
acquisition tools for discovering a behavior that is not yet represented by a
frozen fixture. They are never prerequisites for an ordinary regression run.
After a reviewed acquisition, reduce the observation to a checked-in fixture
and test it offline thereafter.

Large retained JSON/JSONL artifacts are stored as deterministic gzip when their
expanded representation would exceed the repository's raw-artifact budget.
Consumers use `src/compressed-artifact-io.mjs`, so `.json.gz` and `.jsonl.gz`
are storage encodings only: hashes that define evidence identity are computed
from the expanded bytes unless a contract explicitly says otherwise. The test
suite rejects raw verification JSON/JSONL artifacts larger than 8 MiB to keep
new campaign evidence from silently bloating checkouts and repeated I/O.

## Compatibility surface inventory

`scripts/build-compatibility-surface-inventory.mjs` generates `docs/development/compatibility-surface-inventory.json` from the feature catalog, implementation ledger, source coverage, live observations, declared Deepwell JSON-RPC registry, SvelteKit routes and named server actions, Framerail AMC and XML-RPC registries, WWS routes, and the seven audits listed by `open43-blocked-evidence-routing.json`. Inventory v3 keeps the tracked Wikijump inventory content-addressed: it records a stable SHA-256 over the exact registry digest set instead of embedding the Wikijump commit/tree that contains the generated file. FTML remains pinned to its exact external commit/tree, and every registry file read keeps its SHA-256. The pinned `docs/development/compatibility-surface-semantics.json` registry holds the exact 295-record FTML raw denominator, the Catalog-to-FTML crosswalk, closed specification and implementation owner keys, legacy-owner mappings, and the typed edge vocabulary. Raw FTML implementation records remain outside the public feature denominator. Each public surface has one stable identifier, closed owners, typed implementation relationships where evidenced, and independent evidence, source, candidate, standing, and closure fields. The command rejects identity drift, duplicate identifiers or edges, same-count FTML substitutions, Catalog and ledger orphans, missing or extra ownership records, unsupported registry declarations, and values outside the closed vocabularies.

By default the generator reads the current worktree bytes and records their exact registry digest set, which lets source changes and their regenerated inventory land in the same commit. Pass `--source-revision <exact-commit>` when the run must prove those bytes against a committed Git identity or reproduce an older exact commit. Because the tracked v3 bytes no longer contain the Wikijump commit/tree, a metadata-only commit does not require a second inventory rebind commit. After committing an inventory-bearing source head, seal its exact Git identity with `scripts/bind-compatibility-inventory-source.mjs`; the resulting repo-external receipt proves that the committed inventory bytes and every recorded registry digest exist at that exact commit/tree. Post-commit ledger generation consumes that receipt with `--wikijump-binding`, preserving exact source identity without creating a self-referential tracked artifact. The repository `generated-contracts:verify --full` path resolves exact local `HEAD`, regenerates with that revision pinned, and compares the bytes to the tracked inventory.

`scripts/build-deepwell-jsonrpc-contract-manifest.mjs` generates `docs/development/deepwell-jsonrpc-contract-manifest.json` from the current Deepwell JSON-RPC registry and endpoint sources. It records all registered methods with their handler owner, parameter decoder, observed request context requirements, mutation signals, transaction isolation, source identities, and the source-contract test witness. Use `--verify` in the local WSL review/preflight flow to reject a stale manifest. GitHub's offline-compatibility workflow now runs the portable frozen-oracle and product regression layers with network acquisition disabled.

`scripts/build-wws-route-registration-denominator.mjs` generates `docs/development/wws-route-registration-denominator.json` from the production route calls in `wws/src/route.rs`. It requires exactly 30 source registrations, records each declared method class and primary or fallback handler owner, binds every route and handler input to the Git blob and SHA-256 of the exact bytes it parsed, and records four compact source-bound issue #1370 behavior rows with checked public-test anchors plus the Git-bound live observation note. Git identity checks use the admitted absolute `/usr/bin/git` executable with a minimal fixed environment, so inherited PATH and `GIT_*` controls cannot select another repository or object store. The command rejects duplicate registration identities, unsupported router-composition forms, uncommitted source drift, omissions, unsupported route declarations, and handler symbols found only in comments, strings, or test modules. Pass `--verify` to require a byte-for-byte current checked artifact. The earlier 27-route PR 1334 attribution artifact remains immutable historical evidence rather than the current denominator.

```sh
node install/local/wikidot-verification/scripts/build-wws-route-registration-denominator.mjs
```

`scripts/verify-wikidot-py-amc-transport-contract.mjs` verifies the 19 source-contract-only AMC envelope, cookie, transport, redirect, retry, response-status, and exception records in `docs/development/wikidot-py-amc-transport-contract.json`. It binds the contract to the recorded wikidot.py commit, root tree, object IDs, and file SHA-256 values through absolute `/usr/bin/git` with a fixed minimal environment. It rejects omitted, duplicate, unknown, changed, or source-drifted records. Its explicit authenticated-live gaps remain missing; a pass does not claim live AMC parity or Phase 1E completion.

```sh
node install/local/wikidot-verification/scripts/verify-wikidot-py-amc-transport-contract.mjs
```

```sh
pnpm --dir install/local/wikidot-verification compatibility-inventory
```

Pass `--root` and `--output` to run the same source-blind discovery against another repository fixture or to write a temporary comparison artifact.

`scripts/build-compatibility-ledger.mjs` projects that inventory into stable opaque raw, assignment, relationship, and public surface identities. It preserves existing IDs across additions, rejects disappearing raw inputs, records FTML aliases without promoting the 295 FTML implementation records into the public denominator, and keeps evidence, source, test, owner, issue, blocker, candidate, standing, and closure stages separate.

```sh
node install/local/wikidot-verification/scripts/bind-compatibility-inventory-source.mjs \
  --inventory docs/development/compatibility-surface-inventory.json \
  --revision <exact-40-char-commit> \
  --output /absolute/evidence/path/inventory-source-binding.json

pnpm --dir install/local/wikidot-verification compatibility-ledger -- \
  --inventory docs/development/compatibility-surface-inventory.json \
  --wikijump-binding /absolute/evidence/path/inventory-source-binding.json \
  --output /absolute/evidence/path/compatibility-ledger.json
```

## Driftless sandbox oracle

`fixtures/sandbox-oracle-fixture-registry.json` is the checked-in registry for
authored sandbox pages. Its validator enforces the two assertion classes:
`match-live` compares a local browser capture with one frozen Wikidot capture;
`match-frozen-preserved` compares delayed constructs with their declared
preserved local shape and never treats live execution as the expected local
output. ListPages, CountPages, unknown modules, and conditional blocks are
required to use the latter class. The registry schema is
`schemas/sandbox-oracle-fixture-registry-v1.schema.json`.

Compare sealed local and frozen captures without mutating Wikidot:

```sh
pnpm --dir install/local/wikidot-verification sandbox-oracle -- \
  --registry install/local/wikidot-verification/fixtures/sandbox-oracle-fixture-registry.json \
  --local /absolute/evidence/path/local-captures.json \
  --frozen /absolute/evidence/path/frozen-captures.json \
  --output /absolute/evidence/path/sandbox-oracle-verdict.json \
  --run-id sandbox-oracle-20260805
```

The comparison runs exact DOM signature, structure and geometry, computed
style, presence and pseudo-layout, then records screenshot SHA-256 receipts
without a pixel diff. Volatile attribute normalization is declared in the
verdict; when it makes a raw difference disappear the result is the blocking
`normalization_hides_difference` finding, never a pass. The sole exception is
an explicit `environment_identity_translation` for an exact allowlisted live
and local host pair; that event remains in the verdict and does not authorize
arbitrary host normalization.

## Syntax differential runner

The syntax differential runner answers context-free “what does this wikitext render as?” questions by freezing anonymous live Wikidot previews once, then streaming the same syntax cases through one long-lived FTML process. The checked-in starter matrix is `fixtures/syntax-differential/preview-cases.jsonl`.

Capture a new no-replace Wikidot reference file with the repository-pinned `wikidot.py` environment:

```sh
install/local/wikidot-verification/.venv/bin/python \
  install/local/wikidot-verification/scripts/capture_wikidot_preview_references.py \
  --cases install/local/wikidot-verification/fixtures/syntax-differential/preview-cases.jsonl \
  --output /absolute/evidence/path/preview-references.jsonl
```

Routine regression uses Deepwell's exact pinned FTML dependency rather than a separate developer checkout:

```sh
pnpm --dir install/local/wikidot-verification offline:syntax
```

The command builds `deepwell/src/bin/wikidot_syntax_renderer.rs` with Cargo `--offline --locked`, then streams the checked-in `preview-references.jsonl` through that renderer. This keeps the oracle bound to the FTML revision actually used by Wikijump. For diagnostic work against another explicitly chosen FTML executable, the lower-level runner remains available:

```sh
cargo build --manifest-path /path/to/ftml/Cargo.toml --example render_html_jsonl
node install/local/wikidot-verification/scripts/run-syntax-differential.mjs \
  --references /absolute/evidence/path/preview-references.jsonl \
  --renderer /path/to/ftml/target/debug/examples/render_html_jsonl \
  --output /absolute/evidence/path/verdict.json
```

Each syntax case declares a `wikidot_observation_tier`, a `local_execution_tier`, and the page title supplied to both renderers. The current acquisition command accepts only `page-preview`; it performs no authentication or mutation. The current local runner executes only `ftml` cases and reports `wikijump-runtime` cases as `not-applicable`.

The verdict requires parsed DOM tree, DOM signature, and visible text parity. Parsing both fragments before comparison accounts for browser parser behavior such as implicit `tbody` insertion without hiding hierarchy, child order, attribute value, comment, or text-node differences. A mismatch retains both raw HTML fragments for diagnosis. The verdict also records the renderer executable SHA-256 and FTML engine revision. Live references record the resolved Wikidot site and domain, the pinned `wikidot.py` commit and version, and the dependency-file SHA-256.

Review the remaining static mismatches against the checked-in identity-bound policy:

```sh
pnpm --dir install/local/wikidot-verification syntax-dispositions -- \
  --verdict /absolute/evidence/path/verdict.json \
  --policy install/local/wikidot-verification/fixtures/syntax-differential/disposition-policy.json
```

The policy accepts only `intentional-security-boundary`, `wikijump-runtime-boundary`, and `live-observation-resource-failure`, and binds every entry to both its case ID and source SHA-256. An unknown mismatch, changed source, runner error, missing policy case, resolved exception, or policy entry aimed at a `not-applicable` case fails the check. Runtime cases remain outside this exception path: declare `local_execution_tier: wikijump-runtime` and let the syntax runner report them as `not-applicable`, then exercise them through the saved-page runtime lane.

### FTML fixture classification overrides

`scripts/build-ftml-live-pages.mjs` conservatively classifies fixture sources for saved-page, isolated PagePreview, runtime, or not-applicable execution. Its optional `--classification-overrides FILE` argument applies reviewed exceptions from an FTML-owned manifest. Without the argument, classification remains conservative and no override identity is used.

The manifest schema is `ftml.wikidot_parity.classification_overrides.v1` with one `overrides` array sorted by `path`. Every row has exactly `path`, `source_sha256`, `execution_class`, `page_scope`, and one nonempty `reason`. An override applies only when its relative fixture path and lowercase SHA-256 match the source exactly. The loader rejects unknown or missing fields, invalid classes or scopes, duplicate or unsorted paths, stale hashes, and entries that match no collected fixture.

```sh
node install/local/wikidot-verification/scripts/build-ftml-live-pages.mjs \
  --ftml-root /path/to/ftml \
  --classification-overrides /path/to/ftml/tests/fixtures/wikidot-parity/classification-overrides.json \
  --cases-output /absolute/evidence/path/cases.jsonl \
  --pages-output /absolute/evidence/path/pages.jsonl \
  --slug-prefix ftml-parity
```

The JSON stdout summary includes `classification_overrides` as the manifest's resolved path and raw-file SHA-256. It is `null` when the option is omitted, so every reported case-count denominator states whether reviewed overrides influenced it.

Recorded batch-safe cases default to the measured 8,000-character target and 9,000-character hard limit. The 2026-07-26 matrix needed 68 batch requests plus 130 isolated interaction retries at this size. A 20,000-character run needed 33 successful or retry requests plus 173 isolated retries, while a 50,000-character run needed 18 successful or retry requests plus 186 isolated retries; the smaller default therefore minimized total Wikidot requests. Wikidot also returned empty previews for content-dependent 20,000-character and 50,000-character shards, and for every measured 84,528-character and roughly 150,000-character shard. Run `build-failed-preview-retries.mjs` and recapture only failed shards at the default retry size. `compare-wikidot-live-pages.mjs` accepts repeated `--captures` arguments, replaces failed parent shards with successful retries, and fails if any case remains unresolved. Cases whose batched rendering differs from their isolated FTML rendering must then move to the isolated lane; source length alone is not proof that cases are context-independent.

Includes, page-existence checks, permissions, and most runtime modules use the saved-page runtime lane because PagePreview does not execute them. ListPages is an exception: live PagePreview executes site-scoped queries with default-category context but no saved current-page identity. `capture_wikidot_existing_pages.py` reads an existing Wikidot page anonymously and freezes its page ID, latest revision ID and number, public source wikitext and hash, selected rendered subtree, actor state, resolved site and domain, capture time, and pinned acquisition dependencies. Keeping the source bytes makes later corpus-drift diagnosis and exact-input replay possible without another Wikidot request. It accepts only the read-only `scp-wiki`, `scp-jp`, and `sandbox-for-codex` sites and mutates none of them.

```sh
install/local/wikidot-verification/.venv/bin/python \
  install/local/wikidot-verification/scripts/capture_wikidot_existing_pages.py \
  --plans /absolute/evidence/path/saved-page-plans.jsonl \
  --output /absolute/evidence/path/saved-page-references.jsonl

node install/local/wikidot-verification/scripts/run-saved-page-runtime-differential.mjs \
  --references /absolute/evidence/path/saved-page-references.jsonl \
  --runtime-identity /absolute/evidence/path/runtime-identity.json \
  --rerender-receipt /absolute/evidence/path/saved-page-runtime-rerender-receipt.json \
  --local-base https://scp-wiki.wikijump.localhost \
  --local-ca /absolute/path/to/caddy-local-root.crt \
  --output /absolute/evidence/path/saved-page-runtime-verdict.json
```

The runtime verdict compares the selected parsed DOM hierarchy, child order, attribute values, and visible text, then checks required class tokens and forbidden unexpanded directives. It binds the Wikidot source and revision identities to the exact Wikijump SHA, FTML SHA, dependency lock hash, executable or image hash, and runtime configuration hash. Output creation is no-replace. The first canary is the existing read-only `scp-9507` page and its stray-open-bracket include shape from Issue 899.

Before comparing an imported standing page after a new FTML pin, explicitly recompile only the frozen saved-page cases. The operator refuses non-`scp-wiki` references, duplicate slugs, source hashes that differ from live Wikidot, non-loopback RPC endpoints, changed local page or revision identities, and a post-rerender compiler that does not match the runtime identity. It changes compiled artifacts only; page source and revision remain unchanged.

```sh
WIKIDOT_VERIFY_ADMIN_EMAIL=... WIKIDOT_VERIFY_ADMIN_PASS=... \
node install/local/wikidot-verification/scripts/rerender-saved-page-runtime.mjs \
  --references /absolute/evidence/path/saved-page-references.jsonl \
  --runtime-identity "${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing/runtime-differential-identity.json" \
  --rpc-url http://127.0.0.1:12747/jsonrpc \
  --output /absolute/evidence/path/saved-page-runtime-rerender-receipt.json
```

Repeat `--case-id <case-id>` on both the rerender and differential commands to run an explicit source-current subset while retaining a larger frozen reference file. Unknown or duplicate filters fail. The subsequent browser-facing differential requires the source- and revision-bound rerender receipt for the exact selected reference set and exactly one serialized `compiled_generator` whose FTML revision matches the runtime identity. A source-drifted page or stale, missing, or duplicated generator fails even when the selected DOM happens to match.

When a selected public `scp-wiki` reference has drifted from the canonical corpus, build a no-replace refresh bundle from the frozen reference before importing it. The builder preserves the corpus page metadata and entity ID, replaces only the live-derived source, title, revision count, and capture provenance fields, and emits a deterministic receipt plus an import manifest bound to `scp-wiki` and the named corpus branch. It rejects a no-drift selection unless `--allow-no-drift` is given for an intentional metadata-only refresh.

```sh
node install/local/wikidot-verification/scripts/build-saved-page-corpus-refresh-bundle.mjs \
  --references /absolute/evidence/path/saved-page-references.jsonl \
  --case-id scp-7446-stray-open-include \
  --case-id fragment-scp-9988-2-stray-open-include \
  --corpus-root /absolute/path/to/canonical-corpus \
  --branch en \
  --output-dir /absolute/evidence/path/saved-page-refresh-bundle

node install/local/wikidot-verification/scripts/apply-corpus-import-manifest.mjs \
  --manifest /absolute/evidence/path/saved-page-refresh-bundle/import-manifest.jsonl \
  --create-mode db \
  --replace-existing \
  --skip-rerender \
  --skip-attachments
```

After import, run the saved-page rerender command above for the same case IDs and exact runtime identity, then run the HTTPS differential.

### Verify imported source integrity

After materializing corpus pages, compare the selected local page's current revision source, stored corpus snapshot hash, latest import state, and (for selected complex canaries) compiled body length against the exact manifest source files. The command is read-only and exits nonzero for missing pages, changed or blank source, absent/mismatched provenance, incomplete import state, or a required empty body. Select the correct site ID explicitly when the local stack has more than one site:

```sh
node install/local/wikidot-verification/scripts/verify-corpus-import-integrity.mjs \
  --manifest /absolute/path/to/import-manifest.jsonl \
  --site-id 6000006 \
  --db-container wikijump-local-development-database-1 \
  --slug scp-5382 \
  --slug scp-7000 \
  --require-rendered-body scp-5382 \
  --require-rendered-body scp-7000
```

The manifest's source paths are re-hashed before querying the database. The report includes page and revision IDs plus source/provenance booleans, body character counts, and latest import state; it never prints the imported wikitext. The SQL client can instead use `DEEPWELL_VERIFY_DB_URL` or `--db-url` when a direct database connection is appropriate.

Corpus attachment rows use descriptor-bearing direct staging as their canonical import path. Run `apply-corpus-import-manifest.mjs` with `--attachment-create-mode direct`, or use `--skip-attachments` to defer them. The command rejects RPC attachment creation for selected corpus attachments because `file_create` would first commit the current host's libmagic descriptor and its post-commit outdate worker could make that approximation servable before corpus provenance replaced it.

## Standing file descriptor backfill

The file content descriptor migration is additive and nullable so the standing database can migrate without rewriting every file revision in one transaction. A normal standing refresh runs SQL migrations but does not reimport the corpus. The source change can land with descriptor-less Files modules failing closed, but the standing runtime cannot claim Files row completion until `backfill-corpus-file-descriptors.mjs` has a terminal receipt for every active latest revision and every affected public page has rerendered.

Quiesce file mutations before the preflight and keep them quiesced through migration, activation of the new Deepwell binary, and backfill completion. The old binary can leave a finalized retry row in `blob_pending` without a descriptor. Wait for the normal `PrunePendingUploads` job to remove expired rows, then require this command to print exactly `0`; do not delete a live pending row by hand:

```sh
psql "$DEEPWELL_VERIFY_DB_URL" --no-psqlrc --tuples-only --no-align --command "SELECT count(*) FROM blob_pending WHERE s3_hash IS NOT NULL;"
```

Unmoved pending rows with `s3_hash IS NULL` may remain. The new Deepwell binary reads their temporary bytes, derives the descriptor, and persists it before completing the file revision. Apply the complete migration chain only after the moved-pending preflight passes:

```sh
DATABASE_URL="$DEEPWELL_VERIFY_DB_URL" sqlx migrate run --source deepwell/migrations
```

The current source-blind diagnostic is blocked before materialization. Standing `scp-wiki` at site ID 6000006 has 50,301 active latest files totaling 23,139,838,970 bytes. The current `en/by-uuid/*/files/*/snapshots` corpus has 24,464 snapshot JSON files, 24,454 with `mime_description`, totaling 8,505,207,093 descriptor-bearing bytes. The exact snapshot-file-set SHA-256 is `329988eba0e750d33e4ea7ac3556a6e152416089d753286495b20992c85b67f9`. These are pre-run observations, not a completion receipt. The missing corpus provenance must be acquired explicitly; the backfill must not infer the remaining descriptors.

The command always begins with a metadata-only preflight. It requires the sealed `runtime-differential-identity.json` emitted by the exact standing refresh, then records a secret-free live binding for the standing database, Deepwell API, and Files object store. The binding includes immutable container and image IDs, a digest of the effective service configuration, exact loopback publications, the Deepwell config mount, and the protected `wikijump-standing-postgres-data` and `wikijump-standing-files-data` volume mounts. The command verifies the binding before a first run, against the stored receipt on resume, and again immediately before completion. It executes SQL through the sealed database container ID; `DEEPWELL_VERIFY_DB_URL` and `--db-url` are intentionally unsupported.

The metadata preflight seals the exact site pair, active-file baseline, missing-latest and moved-pending checks, deterministic inventory batch hashes, corpus index SHA-256, and corpus snapshot denominator and hash. It records `provenance_matched` and `provenance_missing` before creating an S3 client. Any orphan latest revision, moved pending row without its paired descriptor, inventory-count mismatch, ambiguous corpus candidate, or missing metadata provenance blocks the receipt with SQL staging and public rerenders both at zero. The current corpus therefore blocks in this first phase without reading 50,301 stored objects.

Use the runtime identity from the active standing state home only when it belongs to the activated standing refresh. The default home is `${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing`; it is not a WJLab path. The default standing containers are `wikijump-standing-database-1`, `wikijump-standing-deepwell-1`, and `wikijump-standing-files-1`; pass all three options explicitly if the names differ. Keep the exact site ID and slug guard even when the database container is explicit. For the examples below, set the portable inputs once:

```sh
runtime_home="${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing"
runtime_identity="$runtime_home/runtime-differential-identity.json"
corpus_root="${SCP_CORPUS_ROOT:?set SCP_CORPUS_ROOT to the corpus directory}"
```

This dry run makes no database change and does not rerender pages:

```sh
node install/local/wikidot-verification/scripts/backfill-corpus-file-descriptors.mjs --corpus-root "$corpus_root" --runtime-identity "$runtime_identity" --branch en --site-id 6000006 --site-slug scp-wiki --db-container wikijump-standing-database-1 --deepwell-container wikijump-standing-deepwell-1 --files-container wikijump-standing-files-1 --api-url http://127.0.0.1:12747/jsonrpc --attachment-s3-endpoint http://127.0.0.1:19000 --batch-size 200 --concurrency 16 --dry-run
```

Run the receipt-bearing command even while coverage is incomplete to retain the source-blind preflight evidence. With the current corpus it exits nonzero after writing a `blocked` receipt whose staging and public-rerender counts are zero; it does not enter materialization:

```sh
node install/local/wikidot-verification/scripts/backfill-corpus-file-descriptors.mjs --corpus-root "$corpus_root" --runtime-identity "$runtime_identity" --branch en --site-id 6000006 --site-slug scp-wiki --db-container wikijump-standing-database-1 --deepwell-container wikijump-standing-deepwell-1 --files-container wikijump-standing-files-1 --api-url http://127.0.0.1:12747/jsonrpc --attachment-s3-endpoint http://127.0.0.1:19000 --batch-size 200 --concurrency 16 --receipt /absolute/evidence/path/scp-wiki-file-descriptor-preflight-blocked.json
```

Do not attempt to bypass that receipt or invoke a separate materializer while metadata coverage is incomplete. After an explicit corpus repair, start with a new absolute receipt because the corpus denominator and hash changed. Once metadata coverage is 100 percent, set the 64-hex `DEEPWELL_RPC_TOKEN`, `S3_CUSTOM_ENDPOINT`, `S3_FILES_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_REGION_NAME`, and `S3_PATH_STYLE` in the operator environment. Keep secrets out of process arguments. The same terminal command then byte-verifies every active stored object against its SHA-512 key and exact `(fullname, filename, stored-byte SHA-256)` provenance before it permits any materialization. Run it with the new retained evidence receipt:

```sh
node install/local/wikidot-verification/scripts/backfill-corpus-file-descriptors.mjs --corpus-root "$corpus_root" --runtime-identity "$runtime_identity" --branch en --site-id 6000006 --site-slug scp-wiki --db-container wikijump-standing-database-1 --deepwell-container wikijump-standing-deepwell-1 --files-container wikijump-standing-files-1 --api-url http://127.0.0.1:12747/jsonrpc --attachment-s3-endpoint http://127.0.0.1:19000 --batch-size 200 --concurrency 16 --receipt /absolute/evidence/path/scp-wiki-file-descriptor-backfill.json
```

The three resumable phases are `metadata_preflight`, `byte_preflight`, and `materialize`. The inventory uses a deterministic `(page slug, file ID, latest revision ID)` keyset and holds at most one fixed batch plus the configured number of object bodies in memory. Every inventory and completion transaction first requires the exact active `(site_id, site slug)` pair, so `6000005` cannot silently process `template-en` in place of `scp-wiki`. Byte verification and materialization must reproduce every sealed metadata batch identity. Byte preflight also seals each batch's exact corpus descriptor plan. Only after both complete preflights report zero missing provenance does each batch materialize descriptors in one SQL transaction, rerender every affected saved page, and atomically advance the receipt cursor. Before completion, the command discards the materialization cursor, scans again from the first active file, reproduces every sealed batch identity, and requires every current descriptor to equal its sealed corpus plan. A new latest revision anywhere in the ordering, including behind the old cursor, or a changed descriptor blocks the receipt. A SQL, object, rerender, or receipt failure is safely rerunnable with the identical command; already committed rows are classified as existing and the unadvanced batch is rerendered again.

This standing `scp-wiki` command has one descriptor authority: it treats every active target-site file as corpus-owned and requires an exact `(fullname, filename, stored-byte SHA-256)` corpus `mime_description`. The task-owned `wj-open43-pr2-db` clone has 31 active files and 36 revisions, while every `file.from_wikidot` value is false and every revision comment is empty, so neither legacy field is an import authority. The command never invokes the host `file` program and never infers from MIME, extension, flags, or comments. Any missing or size-mismatched corpus record records a nonzero provenance blocker, blocks the receipt, and requires explicit corpus repair. Existing local uploads are not auto-rescued by this mirror command and remain render-fail-closed when their descriptor is absent; new uploads are owned by the activated Deepwell byte-analysis producer.

A terminal receipt has `status: "done"`, the sealed runtime identity and live resource binding, a complete `completion_inventory` proof, zero `provenance_blockers`, `missing_latest_revision`, `missing_descriptor`, `invalid_descriptor`, and `moved_pending_missing_descriptor`. Metadata and byte preflight rows and matches, completion-rescan rows and descriptor plans, rows completed, singular corpus-provenance authority count, and staging total must each equal the final active-file count, and every sealed batch must be complete before the public rerender receipt advances. The command repeats the full inventory and descriptor-plan scan, aggregate completion checks, site ID/slug guard, and runtime binding check when a completed receipt is presented again. The current 24,454-of-50,301 corpus coverage cannot produce this receipt and remains a blocked preflight observation, not a pass claim. Do not promote the standing runtime while the receipt is blocked or while the served pages have not been verified against the exact activated runtime identity.

## Wikijump identifier leaks

Imported content must carry Wikidot's own DOM names. The Wikidot stylesheet the page loads has no `.wj-` rules, so a leaked `wj-` class is an unstyled element as well as a tree difference.

```sh
pnpm --dir install/local/wikidot-verification wikijump-identifier-leaks -- \
  --site scp-wiki --rpc-url http://127.0.0.1:2747/jsonrpc
```

The check renders a battery of constructs through the local Deepwell runtime by anonymous page preview and fails if any `wj-` class, tag, id, or `data-wj-` attribute appears. The battery covers the constructs FTML renders differently under `Layout::Wikidot` (footnotes, bibliography, collapsible, code, tabview, math, table of contents, user, date, image, video, audio, tables, every link variant, alignment, lists, monospace, raw), the Wikijump-only blocks that must stay literal rather than render (`[[hidden]]`, `[[invisible]]`), and ListPages and CountPages row bodies. A construct that fails to render counts as a failure, since a render error hides whatever it would have emitted.

It requires a running local stack and creates no page. The `--rpc-url` must be loopback: pointing the battery at a remote host would send wikitext off the machine and describe someone else's runtime.

## Corpus-pinned compatibility rules

A compatibility rule earns its place by implementing what Wikidot does, not by recognizing a page the corpus happens to contain. A predicate that compares against a byte-exact fragment of a captured page reproduces that one page and diverges again as soon as a word or a line moves.

```sh
pnpm --dir install/local/wikidot-verification corpus-pinned-literals -- \
  --corpus /absolute/evidence/path/fresh-exact-live-references.jsonl \
  --corpus /absolute/evidence/path/fresh-literal-live-references.jsonl
```

The check is factual rather than stylistic. It extracts every string literal that sits where source text is matched against it, then reports the literals that occur verbatim in no more than eight captured pages. Measured against the 19,612-source campaign corpus, rules pinned to page content occur in one to eight captured pages while genuine module syntax occurs in 321 (`[[collapsible show="`), 658 (`[!--`), 7,803 (`created_at`) and 19,421 (`[[/module]]`); the threshold sits in that fortyfold gap. Pass every lane, because a literal absent from one lane can still be pinned in another. Rust `#[cfg(test)]` modules are skipped, since regression tests are supposed to hold corpus source.

Matches of sixteen characters or more are findings; shorter ones are notices. Short markup vocabulary such as `<br>`, `&quot;`, `[[html]]` and `~~~~` is also rare in a ListPages corpus without having been lifted from any page, and on the current render tree every rare literal at or above that length was genuine page content while every shorter one was vocabulary. Notices still deserve a look, since `属性...` and `@@*@@ ` are short and genuinely pinned; they simply cannot carry a gate on their own. The command reports without failing by default. Pass `--strict` to exit non-zero on findings.

Findings name the exact pages a literal came from. Resolve one by implementing the rule those pages demonstrate, proven by live observation including negative controls, or by leaving the case actionable and recording it as unimplemented. `fixtures/corpus-pinned-literals/allowlist.json` accepts a literal only when the pinned form is genuinely the whole of Wikidot's behavior, and requires at least two live observations per entry.

The check reads string literals only. A rule can overfit through an exact conjunction of ordinary syntax tokens without any corpus-derived literal, so a clean report is not proof that a rule generalizes.

## Generic marker runtime differential

`scripts/run-generic-runtime-differential.mjs` replays the generic saved-page marker captures against an already-running disposable Deepwell runtime. It validates every case, capture, page plan, saved-source hash, marker identity, PagePreview reclassification, and candidate runtime identity before mutation. For each selected capture page it creates exactly one local page, reads its compiled body, compares the marked fragments, and deletes the page in `finally` before moving to the next capture page. The runner does not start or own Docker resources.

```bash
WIKIDOT_VERIFY_ADMIN_EMAIL=... WIKIDOT_VERIFY_ADMIN_PASS=... \
node install/local/wikidot-verification/scripts/run-generic-runtime-differential.mjs \
  --cases /absolute/evidence/path/runtime-cases.jsonl \
  --captures /absolute/evidence/path/runtime-captures.jsonl \
  --external-reference /absolute/evidence/path/runtime-preview-references.jsonl \
  --runtime-identity /absolute/evidence/path/runtime-identity.json \
  --rpc-url http://127.0.0.1:2741/jsonrpc \
  --text-block-url http://127.0.0.1:9000/deepwell-text-blocks/ \
  --output /absolute/evidence/path/generic-runtime-verdict.json
```

Repeat `--captures` and `--external-reference` for multiple artifacts. `--text-block-url` must name the loopback-only anonymous-read endpoint for the disposable `deepwell-text-blocks` bucket. The runner reads each persisted HTML block before comparison, records its ordinal, byte count, SHA-1, and SHA-256, and requires the object to return 404 after page cleanup. Selection is the latest successful fragment capture by `captured_at`; a later failed attempt does not erase an earlier valid observation. Observed differences remain `true-mismatch`; syntax-derived state preconditions are diagnostic hints and never hide an include or module implementation defect. The one explicit exception is a traversal-bearing `[[file ...]]` target: when Wikijump preserves that exact single construct literally while Wikidot emits a link, the report records `accepted-security-deviation`, never `match`. Categories comparisons normalize only the volatile numeric category ID after checking category names and order, toggler/pages/options linkage, and ID uniqueness. Acquisition gaps make the verdict incomplete, while true mismatches and runtime failures fail it. A cleanup failure aborts the run before another page can inherit contaminated state. Output creation is no-replace and credentials are accepted only through the two environment variables shown above.

The source-owned candidate build/manifest contract is documented in [docs/candidate-artifact-binding.md](docs/candidate-artifact-binding.md). `scripts/run-generic-runtime-differential-stack.mjs` is the end-to-end controller. It requires an existing `roku.candidate_build_manifest.v1` dev candidate and never runs Cargo or a build wrapper. This lets every case behind one barrier reuse the same exact Deepwell binary. Before starting the stack, it rechecks the clean repository commit and tree, the manifest bindings, the FTML revisions in `Cargo.toml` and `Cargo.lock`, the Cargo.lock hash, the dev profile and debug artifact path, and the executable's absolute path and SHA-256. It binds `build.binary_path_at_build` by default. Pass `--binary` only to bind a relocated executable with the same hash. The controller starts only a labeled disposable database/cache/files/Deepwell stack, exposes Deepwell and anonymous reads for that run's text-block bucket on separate loopback ports, invokes the runner, saves stack logs next to the verdict, and removes every container, named volume, network, temporary configuration, and loopback listener it created. It does not own or remove the candidate manifest, binary, or target directory. The files bucket stays private. It reads immutable image IDs from the standing containers but never mounts standing volumes. Repeat `--state-fixture` to apply provenance-backed `wikijump_syntax_differential.runtime_state_fixture.v1` JSON artifacts before comparison. Each artifact binds present-page wikitext to SHA-256 and source provenance, deletes declared absent pages, keeps run-owned seed pages for declared active category slugs, and imports provenance-bound Wikidot user identities. `capture-reference-runtime-state-fixture.mjs` accepts repeated `--categories-case` and `--users-case` inputs plus declared absent pages; user imports use the saved-page capture time and reject any inconsistent printuser ID, name, slug, avatar, or karma linkage. The verdict records each input file SHA-256 and every user import, create, edit, delete, category seed, and rerender receipt. State fixtures are rejected outside this disposable-stack controller and can never target the standing runtime through this path.

```bash
pnpm runtime-differential-stack \
  --repository /absolute/clean/wikijump-worktree \
  --candidate-manifest /absolute/evidence/path/deepwell-candidate-manifest.json \
  --cases /absolute/evidence/path/runtime-cases.jsonl \
  --captures /absolute/evidence/path/runtime-captures.jsonl \
  --external-reference /absolute/evidence/path/runtime-preview-references.jsonl \
  --state-fixture /absolute/evidence/path/runtime-state-fixture.json \
  --output /absolute/evidence/path/generic-runtime-verdict.json
```

Run-owned Wikidot mutations remain a separate path. `capture_wikidot_saved_pages.py` permits only `sandbox-for-codex`, only slugs matching `run-owned:ftml-diff-YYYYMMDD-NNN`, and create-only operation followed by identity-checked cleanup. It refuses sources above 160,000 characters or 500,000 bytes, existing slugs, changed cleanup targets, and insecure authenticated transport outside that exact site. Real EN and JP sites remain read-only.

## External candidate cases

`scripts/run-candidate-cases.mjs` attaches one source-owned CandidateCaseSet to an externally owned, sealed, unexpired, non-standing production candidate. CandidateCaseRunner owns pre-run and post-cleanup Docker identity, exact denominator reconciliation, append-only resource registration, normal, error, and signal cleanup, and no-replace per-case and aggregate receipts. It does not build, start, stop, replace, or remove the candidate stack. Browser cases automatically share one user-level, identity-bound persistent external-response evidence cache at `$XDG_CACHE_HOME/wikijump-verification/candidate-public-evidence-v1` (or `~/.cache/wikijump-verification/candidate-public-evidence-v1`). Credential-free public GET/HEAD responses are acquired only for an exact cache miss, retained across campaign/evidence roots, and replayed on later 47-case runs. Before a persistent miss can call the network producer, the cache durably records an acquisition barrier. A successful retained response clears that barrier only after its bytes are sealed; a crash or transport failure after acquisition begins leaves the barrier in place so subsequent runs fail closed rather than automatically issuing the same external request again. Persistent evidence is append-only and is never evicted or discarded merely because an in-memory entry/byte budget would have been exceeded; a response that was successfully acquired must remain retainable rather than forcing the next run to reacquire it. `HEAD`, `Range`, HTTP conditional request headers, and named response `Vary` dimensions are part of the exact replay identity. Legacy `Vary` entries from older manifests are kept without bulk reacquisition but are not used as exact proof until the actually requested anonymous variant has been acquired once and appended with its request-header binding. `Vary: *` is retained after its first observation but is intentionally not replayed as exact evidence; a later proof attempt fails closed without another external request. A retained `304 Not Modified` exchange is preserved but not replayed into a fresh browser cache and is not automatically reacquired. `Set-Cookie` is stripped from retained public responses, while requests carrying Cookie or Authorization or URL userinfo fail closed rather than entering the shared cache. Cross-context and cross-cache-instance misses are serialized before acquisition so the same exact identity is not fetched twice. `WIKIJUMP_CANDIDATE_RESPONSE_CACHE_DIR` and `WIKIJUMP_CANDIDATE_RESPONSE_CACHE_IDENTITY` may still override both values together for a deliberately separate retained evidence identity, and the nested Framerail temporal runner uses that same selected identity rather than opening its own default cache. Candidate-local `.wikijump.localhost` / `.wjfiles.localhost` traffic is exempt from the public network gate. Collapsed local file-mirror source admissions are recorded synthetically without sleeping or honoring an external Retry-After; only an actual external network request increments `external_network_requests`. GitHub CI runs no verifier or candidate tests; the persistent WSL cache is the sole repeated-run acquisition boundary, and a cache hit must contribute zero external network requests.

The `open43-media-files` CaseSet fixes four runtime cases and the editable `scpaiueouiuiuiui` site. It submits the public Framerail multipart action, drives file mutation through public Deepwell JSON-RPC, and observes original and resized identities through public WWS GET and HEAD. Database and filesystem reads are not verdict inputs. The existing generic runtime differential and its disposable-stack controller remain a separate syntax product path.

The `open43-settings-browser` CaseSet fixes nine reversible settings cases in their audited order. It uses the editable `scpaiueouiuiuiui` origin, public Framerail admin actions, public Deepwell JSON-RPC reads, and runner-owned authenticated browser contexts. It records distinct immediate DOMContentLoaded and settled artifacts, including both sides of the category-theme transition, and drives the general settings stale error and successful save through the public browser form. Cleanup restores the pre-run public settings values; it does not create pages or use database or filesystem state as a verdict or cleanup seam. The separate `open43-settings-lifecycle` CaseSet owns the two #758 create cases. Its public cleanup removes only the run-owned pages; the parent run must execute `stop-promotion-candidate.mjs` against the exact candidate stack and retain a passing disposal receipt because the monotonic allocator cannot be restored through a public page-delete seam.

```sh
pnpm --dir install/local/wikidot-verification candidate-cases -- \
  --case-set open43-media-files \
  --candidate-identity /absolute/evidence/path/candidate-parity-identity.json \
  --private-input /absolute/private/path/candidate-cases.json \
  --output-dir /absolute/evidence/path/open43-media-candidate
```

For the settings run, use the same command with `--case-set open43-settings-browser`. Its private input names `deepwell_rpc_url`, `deepwell_rpc_token`, `tls_ca_pem`, and `actors.administrator`, `actors.non_admin`, and `actors.expired`, each with `user_id` and `session_token`. It also names `fixture.site_id`, a `fixture.cross_site_sentinel_id` that a public preflight proves does not resolve, and `fixture.default_category` and `fixture.transition_category`; each category supplies `category_id`, `slug`, `page_id`, and `page_slug` for the public category-transition check. The throttle receipt binds only hashes of that private identity and never stores a cookie or storage state.

The candidate identity must seal `scpaiueouiuiuiui.wikijump.localhost` and its matching files origin at one non-443 loopback endpoint. For the media run, the private JSON names `deepwell_rpc_url`, `deepwell_rpc_token`, `object_store_origin`, `presigned_origin`, `tls_ca_pem`, and `actors.editor.user_id` plus `actors.editor.session_token`. Its Deepwell and object-store URLs must be explicit loopback publications of the sealed candidate Compose services. The private input must be one regular file with no group or other permissions, and the output directory must not already exist. Receipts record the input file SHA-256 and hashes of its token, session, and CA values, never the raw values or a presigned URL.

`scripts/provision-candidate-account.mjs` prepares an existing imported Wikidot identity only on that same disposable editable candidate. It verifies the mode-0600 private input is sealed to the exact candidate identity SHA-256, verifies its operator session resolves to platform user `-1`, and treats only the safe integer ID plus the exact imported public `user_get` name, slug, and user type as Account A identity authority. It calls `user_activate_from_wikidot` for the first activation or `user_edit` for an exact already activated identity, and uses the public `member_get` and `member_set` methods to add only missing `scpaiueouiuiuiui` membership. It does not grant a role, replace existing membership, call Wikidot, or accept a site selector.

```sh
pnpm --dir install/local/wikidot-verification candidate-account-provision -- \
  --candidate-identity /absolute/evidence/path/candidate-parity-identity.json \
  --private-input /absolute/private/path/candidate-account.json \
  --receipt /absolute/evidence/path/candidate-account-receipt.json
```

The private JSON names `candidate_identity_sha256`, `deepwell_rpc_url`, `deepwell_rpc_token`, `tls_ca_pem`, `operator.user_id`, `operator.session_token`, `account.wikidot_user_id`, `account.public_name`, `account.public_slug`, `account.login_identifier`, `account.password`, and `account.locales`. The externally owned disposable candidate lifecycle is the operator-session generation owner: it creates a short-lived session for the seeded platform user `-1` through its private fixture setup and seals that session into this input; this command neither manufactures a platform session nor accepts one from an argument or environment variable. `operator.user_id` must be `-1`, Account A's ID must be a safe integer distinct from `-1`, and `account.login_identifier` must equal the exact imported public name or slug. The command proves the correct and different password behavior through the ordinary public `/-/login` action, logs out every probe session, and publishes a mode-0600 no-replace receipt containing only candidate and private-input hashes, the public numeric/name/slug identity, a login-identifier hash, site membership outcome, and login status observations.

The 2026-07-26 benchmark streamed 10,000 frozen matching cases through one debug FTML renderer process and completed the local render and comparison in 1.72 seconds with 302,324 KiB maximum RSS. Treat this as a throughput baseline, not a fixed performance gate.

## Identity-bound differential runner

`scripts/run-identity-bound-differential.mjs` is the thin one-command owner for one complete saved-page runtime differential case. Its only supported adapter runs the existing candidate-manifest-bound `run-generic-runtime-differential-stack.mjs` interface. It accepts no executable or argument extension points.

```sh
pnpm --dir install/local/wikidot-verification identity-bound-differential -- \
  --case-manifest /absolute/evidence/path/case-manifest.json
```

The `wikijump_syntax_differential.case_manifest.v1` case manifest uses absolute paths. It binds exactly one runtime case source, an independent list of site-state fixtures, saved Wikidot captures and external references, the exact saved Wikidot page URL, the seeded administrator actor, saved-page context, the candidate build manifest, this repository, the Node executable running the outer command, fixed Git and Docker executables, the stack report path, and the final verdict path. The selected capture domain and slug must match that URL. The outer runner and stack controller invoke that same absolute Node path without PATH lookup. The stack controller verifies that the clean repository HEAD, FTML Cargo pin, dependency lock, and executable hash match the candidate manifest before it starts its disposable runtime. It runs absolute Git and Docker paths with minimal environments and a fixed local Docker socket. The served Deepwell process is that bound executable, not an arbitrary standing URL.

Raw HTML, parsed DOM, and visible text are mandatory. Browser intervals may be not applicable only with a reason whose basis is the case contract. Stack arguments are derived only from the validated manifest. The stack controller owns its disposable Compose project, run root, runtime pages, and state, then writes a cleanup receipt; a missing or failed receipt makes the outer verdict fail. The outer command has no timeout that can interrupt controller cleanup. The final `wikijump_syntax_differential.identity_bound_verdict.v1` file is published without replacement and records the actual Node invocation plus absolute paths and SHA-256 values for every retained input, runtime report, cleanup receipt, and stack log when Compose started. The report, cleanup receipt, stack log, and final verdict paths are all reserved before execution and never replaced. A moving identity, omitted channel, incomplete case, partial report, failed cleanup, unknown manifest field, or output collision fails closed.

## Python environment

The authenticated Wikidot helper runs from this component's private `.venv`. The runtime package subset mirrored from the frozen `requirements.txt` manifest lives in `requirements-pypi.txt`, build packages live in `requirements-build.txt`, and both are hash-locked in `requirements.lock`; the owner's `Rokurolize/wikidot.py` fork remains pinned separately to a full commit in `requirements.txt`. The setup script installs the hash-verified packages first, fetches only that commit, verifies the checked-out `HEAD`, and installs it without dependency resolution or build isolation. Create or refresh the environment before using the theme-localization execution path:

```sh
install/local/wikidot-verification/scripts/setup-python-env.sh
```

The helper never imports from a mutable host checkout. Credentials remain environment-only inputs to the helper process.

Theme-localization execution is site-scoped by the audited two-member allowlist
(`scpaiueouiuiuiui` and `sandbox-for-codex`), mirrored in JavaScript and the
Python helper and checked at every origin boundary. The default site remains
`scpaiueouiuiuiui`; selecting the sandbox requires explicit
`--site sandbox-for-codex --wikidot-origin https://sandbox-for-codex.wikidot.com`
and `--wikijump-origin https://sandbox-for-codex.wikijump.localhost`. The
allowlist widening is a separate draft change held for owner sign-off before
any live sandbox mutation.

## Completion controller

`scripts/run-completion-controller.mjs` is the resumable one-command entry point for a complete branch run. It executes an explicit JSON plan without a shell, records every command through the command ledger, hashes declared inputs and outputs, checks declared verdicts, and writes compact state and summary files.

```sh
node install/local/wikidot-verification/scripts/run-completion-controller.mjs \
  --plan /absolute/path/completion-plan.json \
  --state /absolute/path/completion-state.json \
  --summary /absolute/path/completion-summary.json
```

The plan uses schema `wikijump_full_parity.completion_plan.v1`. Paths are resolved relative to the plan file. Every stage declares one or more regular-file evidence outputs, and each output has exactly one owning stage. Verdict files and root-cause cluster files must also be declared as outputs. The first output of the required manifest stage is the frozen manifest recorded in the terminal summary.

A diagnostic plan may run a bounded prefix or probe. A complete plan must contain the following dependency-ordered stage kinds:

1. Exactly one `freeze_manifest` or `consume_manifest` stage.
2. `import`.
3. `render`.
4. `browser_capture` and `browser_replay` for the two immutable-candidate passes.
5. `compare`.
6. `workflow` and `client` in either order.
7. `certify`.

Complete plans also bind `candidate.wikijump_sha`, `candidate.ftml_sha`, `candidate.artifact_key`, `candidate.runtime_identity_sha256`, and `candidate.runtime_config_sha256`. Keep credentials out of the plan. Commands inherit the controller environment, while sensitive command arguments are redacted by the command ledger.

Minimal diagnostic plan:

```json
{
  "schema": "wikijump_full_parity.completion_plan.v1",
  "run_id": "en-merged-head-20260718",
  "branch": "en",
  "mode": "diagnostic",
  "ledger_path": "./command-ledger.jsonl",
  "stages": [
    {
      "id": "consume-manifest",
      "kind": "consume_manifest",
      "command": "node",
      "args": ["verify-frozen-manifest.mjs"],
      "cwd": ".",
      "inputs": ["./source-lock.json"],
      "outputs": ["./verified-source-lock.json"],
      "timeout_ms": 30000
    }
  ]
}
```

Resumption is fail-closed. A stage is reused only when the exact plan bytes, command contract, input hashes, dependency receipts, output hashes, and verdict file all match the passing receipt. Mutated or missing evidence reruns the stage. A same-host lock whose recorded process no longer exists is recovered after inode verification; live or ambiguous locks remain blockers.

For root-cause reduction, a stage may declare `cluster_sources`. JSON or JSONL records are deduplicated by the configured `key_fields`, with occurrence counts and source-stage provenance retained in the terminal summary.

## XML-RPC pilot local comparison

`scripts/compare-xmlrpc-pilot-local.mjs` accepts only the designated sealed 128-page XML-RPC pilot source, turns it into a verified pilot manifest, and compares its live rows with an already-running local Deepwell runtime. It makes no Wikidot request and sends only unauthenticated loopback `site_get` and `page_get` calls. The runtime identity input must carry the exact Wikijump and FTML SHAs, artifact key, and runtime configuration SHA.

```sh
node install/local/wikidot-verification/scripts/compare-xmlrpc-pilot-local.mjs \
  --pilot-root /mnt/oracle-store/wjlab/xmlrpc-pilot-en-128-... \
  --runtime-identity /evidence/runtime-identity.json \
  --rpc-url http://127.0.0.1:12747/jsonrpc \
  --output-dir /mnt/oracle-store/wjlab/xmlrpc-pilot-local-comparison-...
```

The output directory receives a no-replace verified pilot manifest, local comparison rows, mismatch clusters, and `xmlrpc-pilot-verdict.json`. Live rows compare exact source, compiled HTML, revision count, and timestamp instant. A typed `wikidot_deleted` tombstone remains a neutral source-state observation: it is never converted to blank source or HTML and does not cause a local page lookup. A rerun recomputes read-only local observations and accepts already-sealed output files only when their bytes are identical.

## Read-only browser capture

`scripts/capture-browser-rendering.mjs` uses a fixed host-wide capture lock and durable request-gate state under `/var/tmp/`. External evidence is cache-first: identity-bound retained responses are replayed without another public request, a genuine cache miss is acquired once and persisted for reuse, and the fixed inter-request interval is 0 ms. The source and local browser contexts share the same persistent response cache; exact local origins stay exempt, while any incidental external dependency emitted by the local page must replay from that cache or perform the one guarded acquisition. Anonymous document caching and exact `Vary` binding are enabled by default so omitting an optional cache flag cannot silently revert a repeated run to in-memory-only acquisition. An explicit server `Retry-After` deadline is still persisted and honored for cache misses. Service workers and WebSockets are blocked. The command accepts only canonical standing `https://<site>.wikijump.localhost` page URLs as local exemptions and derives the matching `https://<site>.wjfiles.localhost` file origin; public or credentialed inventory values fail before browser startup. It seals `request-gate-config.json` before starting the proxy or browser and records final gate counters in `records.json`. A failed state confirmation leaves the lock pending and blocks a later capture until an operator reviews it.

For anonymous read-only source inspection, `--source-only` skips the local context. An inventory row may declare `capture_states`, each with a stable `state_id`, viewport, and actions. The state runner permits only search-field fill, focus, hover, scroll, safe hash targets, and clicks on an explicit list of non-mutating navigation/tab/collapsible/credit controls. It reloads the same source URL for each state and retains the final DOM, screenshot, status, action result, and hashes. `blocked_source_request_prefixes` may exclude exact HTTP(S) host/path prefixes that are already known to be outside the captured contract; every excluded request is recorded on the evidence row. Do not use those exclusions to hide required platform markup, site CSS, or first-party behavior.

## Redirect runtime reproducibility

`scripts/validate-redirect-runtime.mjs` validates corpus-provenanced redirect routes without following them or contacting their destinations. It requires the full inventory, the sealed real-Wikidot status and `Location` authority, the frozen corpus redirect inventory, and the exact local runtime identity. The validator reconciles all three fixture sets, requests every route twice through an explicit loopback address, and requires exact status, `Location`, header multiplicity, body hash, and body size reproducibility.

```sh
node install/local/wikidot-verification/scripts/validate-redirect-runtime.mjs \
  --inventory /evidence/full-inventory.json \
  --authority /evidence/redirects-real-wikidot.json \
  --corpus-redirects /evidence/redirects-frozen-corpus.json \
  --runtime-identity /evidence/runtime-identity.json \
  --local-base https://scp-wiki.wikijump.localhost \
  --resolved-address 127.0.0.2 \
  --output /evidence/redirect-verdict.json \
  --document-inventory-output /evidence/browser-document-inventory.json \
  --ignore-https-errors
```

Only an explicit loopback IP is accepted. Redirects are never followed, so an external `Location` remains observable evidence rather than an outbound browser or HTTP request.

The document inventory output is the exact complement of the sealed redirect set. The verdict records full, redirect, and document counts plus deterministic fixture-set hashes, so redirect routes and normal browser documents can be validated by separate surfaces without a manual queue or silent omissions.

## Standing candidate browser parity

`scripts/run-standing-browser-parity.mjs` defines the source-owned browser-parity receipts used around promotion. `verify-standing-candidate-parity-admission.mjs` binds candidate evidence to the candidate identity, live reference, and completion policy; `install/standing/scripts/verify-promotion-precondition.mjs` binds that admission to the sealed build, final-frozen inputs, and staging home before standing preparation or refresh. The runner has three intentional modes. `live-reference` captures only the six production-theme canaries from `scp-wiki.wikidot.com` through an identity-bound persistent response cache with a 0 ms fixed inter-request interval; repeated runs reuse retained responses, while cache misses still honor server `Retry-After`. `candidate` captures the same pages only from a sealed, expiring non-443 candidate and compares them to an exact sealed live reference. `standing` runs only after a passing standing refresh receipt, captures the canonical local `https://scp-wiki.wikijump.localhost` and `https://scp-wiki.wjfiles.localhost` authorities on port 443 without rewriting their origins, replays retained source redirects through the same evidence cache, and seals a post-refresh browser proof bound to that refresh receipt. Candidate mode never targets port 443; standing mode intentionally does because changing the standing origin or port changes CSP and resource semantics. Historical sealed references retain the request interval recorded when they were acquired and remain reusable evidence rather than forcing another external fetch.

Any later merge that changes runtime inputs after a passing final-zero makes that receipt historical. Re-certification must establish a fresh pre-merge candidate identity from the new runtime base, then repeat promotion, standing proof, reconciliation, and final-zero; do not carry a previous standing or final-zero verdict across a dependency-only runtime merge.

```sh
node install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs \
  --mode live-reference \
  --output-dir /mnt/oracle-store/wjlab/standing-live-reference-... \
  --live-completion-policy /secure/standing-live-completion-policy.json \
  --source-response-cache-dir "$HOME/.cache/wikijump-verification/standing-live-reference-v1" \
  --source-response-cache-identity standing-live-<browser-sha>-<policy-sha> \
  --cache-source-documents \
  --browser-root framerail \
  --browser-executable /usr/bin/google-chrome
```

```sh
node install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs \
  --mode candidate \
  --output-dir /mnt/oracle-store/wjlab/standing-candidate-parity-... \
  --live-completion-policy /secure/standing-live-completion-policy.json \
  --candidate-identity /secure/candidate-parity-identity.json \
  --live-reference-ledger /mnt/oracle-store/wjlab/standing-live-reference-.../standing-browser-live-reference.json \
  --live-reference-sha256 <sealed-reference-sha256> \
  --browser-root framerail \
  --browser-executable /usr/bin/google-chrome
```

```sh
node install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs \
  --mode standing \
  --output-dir /mnt/oracle-store/wjlab/standing-browser-parity-... \
  --live-completion-policy /secure/standing-live-completion-policy.json \
  --standing-refresh /secure/standing-refresh.json \
  --live-reference-ledger /mnt/oracle-store/wjlab/standing-live-reference-.../standing-browser-live-reference.json \
  --live-reference-sha256 <sealed-reference-sha256> \
  --source-response-cache-dir "$HOME/.cache/wikijump-verification/candidate-public-evidence-v1" \
  --source-response-cache-identity wikijump-candidate-public-evidence-cache-v1
```

The live policy is sealed before any browser request and names each tolerated external failure exactly. A candidate identity is sealed before local capture and binds its repository/tree, FTML pin, immutable image IDs, isolated configuration hashes, owner/expiry, non-443 endpoint, loopback address, and evidence seal. Before opening the browser, and again after browser, proxy, request-gate, and lock closure, the runner independently inspects the candidate Compose project. Every declared role must be running exactly once with the sealed image, provenance labels, expiry, artifact key, configuration hash, and a Caddy HTTPS mapping limited to the declared loopback non-443 endpoint. The identity also carries the expected aggregate hash of effective Docker service configuration. The runner computes that hash from command, entrypoint, environment, mounts, network, port, and security settings without recording secret values. Candidate mode runs only from a clean source checkout whose exact Wikijump tree and FTML lock pin match the candidate, and binds a canonical manifest of every parity module into the receipt. Standing mode instead binds the proof to the validated refresh receipt and keeps both canonical local HTTPS origins unchanged; local file redirects are resolved through retained evidence before Chromium follows them so a missing mirrored file does not turn a source response into a different cross-origin browser observation. The resulting receipts reject a missing canary, stale or mismatched source identity, local-only anomaly, omitted screenshot, incomplete load/font/image observation, or a record that lacks the `DOMContentLoaded` observation. A terminal ledger and receipt are published only after clean closure, with the final shared-gate snapshot bound into both. The immediate capture is DOM/CSS evidence at `DOMContentLoaded`, not a compositor-filmstrip claim.

For a direct Framerail candidate that is not behind WWS, `--site-id ID` injects the non-secret trusted routing identity for the fixed `scp-wiki` authority. Omit it when exercising the complete edge path.
