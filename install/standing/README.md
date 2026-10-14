# Standing runtime operations

`compose.yaml` is the source-owned topology for the browser-facing standing runtime. The rendered home uses the role-based `wikijump-standing` project and the durable external volume names `wikijump-standing-postgres-data`, `wikijump-standing-files-data`, `wikijump-standing-cache-data`, `local-caddy-data`, and `local-caddy-config`. Never run `docker compose down -v` against this topology. Legacy `runtime50x-*` volume names are migration-only rollback inputs and are not part of the active topology.

The durable cache has a source-owned compatibility floor in
`cache-data-compatibility.json`. A standing/cache image must not be downgraded
below that Valkey version: persisted AOF/RDB data may use formats introduced by
the floor release. The hermetic compatibility suite checks both local/dev
Dockerfiles against that policy so a future base-image cleanup cannot silently
turn a standing activation into an unreadable-cache rollback incident.

## Startup topology

Warm startup latency was dominated by Docker bridge/NAT and published-port
programming per container, not by application initialization. In this topology
only Caddy is attached to the Docker bridge and publishes host ports
(`80/tcp`, `443/tcp`, `443/udp`). Database, files, cache, Deepwell, Framerail,
and WWS share Caddy's network namespace with `network_mode: "service:caddy"`, so
the engine programs one network endpoint instead of seven.

Inter-service traffic therefore uses `127.0.0.1`, not Compose service DNS, and
Caddy resolves the application upstreams through `extra_hosts`. The application
services no longer expose host ports; inspect them through the namespace (for
example `docker compose exec caddy ...`). Recreating Caddy restarts the services
that share its namespace, so a Tier 1 refresh recreates only Deepwell,
Framerail, and WWS and never stops Caddy.

Caddy, Framerail, and WWS run with `WIKIJUMP_STARTUP_READINESS=true`: they serve
an explicit `Cache-Control: no-store` 503 until a real authenticated Deepwell
`ping` RPC succeeds, then Caddy atomically loads the Deepwell-generated routes.
Deepwell keeps its `depends_on` on the base services so migrations run exactly
once against a healthy database, and it retries only transient Redis/Valkey
startup unavailability (`BusyLoading`, connection refusal) in-process instead of
exiting into a Docker restart loop. Startup 503s are acceptable; partial HTML,
stale application routes, and false 200s are not.

There are two operational tiers. Routine application refreshes use Tier 1. Tier 2 is reserved for operations that can change a named volume attachment, the Compose project name, the network name, or edge routing.

## Tier 1: routine merged-head refresh

Tier 1 is the expected default after a change merges to `develop`. It activates the immutable Deepwell, Framerail, and WWS images that already passed the sealed pre-merge candidate. Preparation verifies and binds those exact image IDs to the normal two-parent merge whose tree equals the candidate tree; it does not rebuild them. Activation never compiles Rust or bundles JavaScript. It does not stop Caddy, database, files, or cache; it does not run `down`; and its command-line parser has no volume-removal or Compose passthrough option.

A Tier 1 refresh is required before anyone asserts that a browser-visible, chrome, layout, or DOM defect is fixed or still present in the standing runtime. A browser claim against an older standing SHA is not evidence about current code.

Start from a clean checkout whose `HEAD` equals the fetched `origin/develop` head. Bind the three exact candidate application images to that merged source:

```sh
git fetch origin develop
runtime_home="${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing"
mkdir -p "$runtime_home"
python install/standing/prepare.py \
  --source-root "$PWD" \
  --promotion-precondition /secure/promotion/promotion-precondition.json \
  --output "$runtime_home/prepared-<wikijump-sha>.json"
```

Then activate that receipt without a build:

```sh
python install/standing/refresh.py \
  --source-root "$PWD" \
  --runtime-home "$runtime_home" \
  --prepared-receipt "$runtime_home/prepared-<wikijump-sha>.json"
```

Preparation records the candidate-image verification duration and exact image IDs. Activation performs one fixed sequence:

1. Verify the source checkout is clean and exactly matches `origin/develop`, then read the exact Wikijump tree and FTML pin.
2. Verify that the promotion candidate is a parent of the normal two-parent merge, its tree and FTML pin equal the merged source, and the prepared receipt reuses the exact sealed candidate image IDs. Verify the production Dockerfile hashes, immutable references, profiles, and candidate provenance labels, then inspect each local image ID again immediately before activation.
3. When saved pages use a different renderer generation, run `render_ahead.py` to completion against the prepared immutable Deepwell image while the old version keeps serving. Bind the passing image/generator receipt to the refresh; a failure prevents activation. When saved pages are already current, skip this step.
4. Atomically update the three `STANDING_*_IMAGE` values, `STANDING_WIKIJUMP_SHA`, `STANDING_FTML_SHA`, and the prepared resource expiry in the runtime `.env`. Deepwell reads the locales packaged in its immutable production image.
5. Run `docker compose --project-name wikijump-standing up --detach --no-deps --no-build deepwell framerail wws` with the checked-in refresh label overlay. The overlay adds owner and expiry labels to the three recreated containers and has no volume declarations.
6. Wait for all three services to become healthy, fetch `http://scp-wiki.wikijump.localhost/scp-9506`, require the expected document markers, update `runtime-differential-identity.json`, and seal a revision-specific `refresh-receipt-<wikijump-sha>.json` with the preparation receipt, exact source, FTML pin, dependency lock, render-ahead decision and receipt, application image IDs, effective Compose configuration, phase timings, health, canary, and resource-disposition record.

### Renderer generation switches: render ahead

Deepwell refuses saved output whose `compiled_generator` is not its own (`ftml vX; deepwell-render/vN`), so a release that bumps the renderer epoch or FTML makes every saved page stale at activation. Rerendering after activation means the site is unavailable for the whole rerender (about two hours for the standing corpus). `refresh.py` now invokes render-ahead automatically when the saved-page generator differs, while the old runtime keeps serving. For diagnostics or an intentional manual pre-render, run:

```sh
python install/standing/render_ahead.py \
  --prepared-receipt "$runtime_home/prepared-<wikijump-sha>.json" \
  --output-dir "$runtime_home/render-ahead-<wikijump-sha>"
```

The command runs the prepared Deepwell image's `render-ahead` runtime action as a one-off container in the serving Deepwell's network namespace, with its environment and configuration. It creates the `page_revision_render_ahead` table from the image's idempotent migration without recording the migration, so the serving build (and a rollback to it) still passes its own migration check. Each stale page's latest revision is rendered into that table; `page_revision` is untouched. Rendering refreshes link rows, and query modules read them. After each full pass, the pages whose link content (endpoints, type, count; not row timestamps) changed are rendered once more: a page that changes again links differently on every render (random ListPages, #2435) and is recorded in the receipt; if any other page changed, every page is rendered again, up to three full passes. `--rerender-all` ignores rows from an earlier run that could not establish this. Pages that miss the serving render deadline under load are retried alone, then once with a one-shot `--tail-render-timeout-ms` deadline. The receipt `render-ahead-receipt.json` records the image, generator, per-pass timings and counts, and `stale_without_valid_ahead`, which must be `0`.

The new Deepwell promotes valid rows in one statement when it starts, before it accepts requests. A row is valid only while its revision is still the page's latest, with the same source, and has not been rerendered by the serving build since the row's inputs were read. Pages that changed after the run therefore stay stale. `refresh.py` verifies the required pre-render before parking any container and rechecks `stale_pages_remaining: 0` after activation; a failure triggers the existing rollback path. The renderer switch therefore needs only the container restart when the saved-page inputs have remained valid.

The candidate Framerail image is built from the production Dockerfile with origin checks enabled by default, and the standing Compose environment repeats `FRAMERAIL_CSRF_CHECK_ORIGIN=true` so the runtime cannot silently disable the check.

The standing image tier is deliberately separate from `install/local`: local images retain bind mounts and watch-mode startup for source iteration, while the sealed candidate images are already-built production artifacts. Deepwell's candidate image contains the pinned `sqlx-cli` and migrations and runs `sqlx migrate run` before the binary; migration handling is therefore explicit rather than silently lost when the local startup script is removed.

The script refuses unknown arguments, including `-v`, `--volumes`, and `--remove-volumes`. There is no argument that is forwarded to Docker or Docker Compose.

## Rendering the canonical home

`render.py` materializes the canonical home from a clean checkout at an exact merged Wikijump revision. Export `DEEPWELL_RPC_TOKEN` as the 64-character lowercase hexadecimal service credential before rendering; the renderer validates it and stores it only in the mode-600 runtime `.env`, never in `identity.json` or image arguments. It copies the production Deepwell config and replaces only the production domain pair with `wikijump.localhost` and `wjfiles.localhost`. Deepwell uses the locales already packaged at `/opt/locales` in the immutable production image, so the rendered home contains no checkout path or external locales bind. The renderer fails closed if the production domain block changes, the requested FTML revision is absent, either required Deepwell runtime source is missing, or the checkout identity is not exact. `identity.json` records the source tree, FTML pin, config hash, image inputs, and persistent volume names without host paths or render-time fields.

`identity.json` describes the materialized topology and is updated only by `render.py`. `runtime-differential-identity.json` describes the application artifacts currently activated by the latest successful Tier 1 refresh and is the identity input for saved-page runtime comparisons.

Rendering does not mutate the running stack. A routine Tier 1 refresh uses the already materialized home; a topology change uses Tier 2.

## Tier 2: topology, volume, network, or edge maintenance

Tier 2 is the controlled maintenance ceremony for changes to named volume attachments, the Compose project name, the network name, or edge routing. Routine source updates do not use Tier 2. The old `runtime50x-*` identities must never be introduced into a newly rendered standing home.

1. Record the current container identities, image digests, mounted volumes, network, edge owner, and passing canaries as rollback evidence. Verify the five named persistent volumes and their consumers before changing anything.
2. Build the exact merged-head images under the required lease, render a staging home bound to those identities, and validate it before touching the canonical home.
3. Activate the bounded explicit 503 maintenance response. Connection refusal, a generic 5xx, or silently serving an old candidate is not an acceptable maintenance state.
4. Stop the old standing containers without `-v`, start the reviewed topology, and verify that every named volume survives. Never prune or recreate a named data volume as part of this ceremony.
5. Verify one port-443 owner, expected project and network identities, service health, HTTP and assets, WIKIREQUEST metadata, AJAX ListPages, DOM, and unmodified `wikidot.py` lookup canaries.
6. On failure, restore the saved topology without `-v`, restore normal edge routing, and seal a failed receipt. On success, seal the promotion receipt, retain the immediate rollback image, and remove only superseded images whose lifecycle closure is proven.

Physical volume migration is explicit and rollback-preserving. Copying data to a durable name and changing the Compose attachment are separate operations; neither is an incidental effect of Compose recreation. The migration verifier must prove checksum-equivalent copies while the old volumes are quiesced before the active topology is switched.

### One-time legacy volume migration

The campaign-era `runtime50x-*` names are migrated exactly once. Stop the
standing project without removing containers or volumes, then copy and verify
all three data volumes while they are quiesced:

```sh
runtime_home="${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing"
mkdir -p "$runtime_home"

docker compose -p wikijump-standing -f /path/to/current/standing/compose.yaml stop

python install/standing/volume_transfer.py migrate \
  --receipt "$runtime_home/legacy-volume-migration.json"
```

`volume_transfer.py migrate` refuses to run while any `wikijump-standing`
container is running. It resolves the currently installed Deepwell image as a
network-disabled copy/verification helper, creates only the durable
`wikijump-standing-{postgres,files,cache}-data` destinations, copies with
numeric ownership/ACL/xattr preservation, and computes the same deterministic
tree digest for the legacy source before copy, the destination after copy, and
the legacy source again after copy. A mismatch fails closed. The legacy volume
is never deleted by the migration command. If an interrupted run leaves a
partial durable destination, inspect its consumers first and rerun explicitly
with `--replace-destinations`.

After the verified copy exists, activate the reviewed/merged standing topology
whose Compose file names the durable volumes. Then prove that the running
database, files, and cache containers mount only the durable names while all
three legacy rollback inputs still exist:

```sh
python install/standing/volume_transfer.py post-cutover \
  --receipt "$runtime_home/legacy-volume-migration.json" \
  --output "$runtime_home/legacy-volume-cutover.json" \
  --runtime-home "$runtime_home"

pnpm --dir install/local/wikidot-verification run offline:browser
```

Also run the ordinary standing health, SCP-9506, file/object-store,
database/cache, WIKIREQUEST, AJAX ListPages, and `wikidot.py` canaries. Only
after those pass and `docker ps -a` shows no container mounting a
`runtime50x-*` volume may the three legacy volumes be removed. Their removal is
an explicit operator action; neither Compose nor `volume_transfer.py` performs
it implicitly. The optional `--runtime-home` binding is part of the cutover
proof: it refuses a Compose file that still names a legacy volume and updates
only `identity.json`'s persistent-volume identity plus the migration receipt
hash, so an older rendered home cannot keep advertising the retired names.

### Portable backup and clean-Ubuntu restore

The same tool is the repository-owned state-transfer path for a fresh Ubuntu
WSL instance. A consistent backup includes the three durable application data
volumes plus `local-caddy-data` and `local-caddy-config`, preserving the local
Caddy CA as well as Wikijump data. Quiesce the standing stack first:

```sh
helper_image="$(docker inspect -f '{{.Image}}' wikijump-standing-deepwell-1)"
docker compose -p wikijump-standing -f "$runtime_home/compose.yaml" stop

python install/standing/volume_transfer.py backup \
  --output-dir /absolute/backup/wikijump-standing \
  --helper-image "$helper_image" \
  --gzip

docker compose -p wikijump-standing -f "$runtime_home/compose.yaml" start
```

`manifest.json` seals each archive SHA-256 plus the deterministic extracted
tree digest. Keep that directory outside the disposable checkout/WSL instance.
On the clean Ubuntu instance, install Docker, clone the exact desired Wikijump
source, make the recorded (or another compatible GNU-tar) helper image locally
available, and restore before starting standing:

```sh
python install/standing/volume_transfer.py restore \
  --input-dir /absolute/backup/wikijump-standing \
  --helper-image sha256:<local-helper-image-id>
```

Restore refuses pre-existing destination volumes, verifies every archive hash
before creating any destination, then verifies each extracted tree. Render the
standing home from the clean checkout into
`${XDG_STATE_HOME:-$HOME/.local/state}/wikijump/standing`, activate it with the
normal merged-head promotion path, and rerun the same standing/browser
canaries. No `/home/roku/wjlab` path is required; an old WJLab tree is archive
provenance only.

The archive can also be rehearsed non-destructively on the source host while
the canonical durable volumes still exist. Supply a temporary lowercase Docker
volume prefix ending in `-`; the archive still has to name the canonical source
volumes, but extraction and tree verification use prefixed destinations:

```sh
python install/standing/volume_transfer.py restore \
  --input-dir /absolute/backup/wikijump-standing \
  --helper-image sha256:<local-helper-image-id> \
  --volume-prefix restore-rehearsal-
```

After a successful rehearsal, remove only those explicitly prefixed temporary
volumes. This is the same archive-hash, extraction, ownership/xattr, sparse-file,
and deterministic-tree verification path used by a clean-host restore; it does
not read from the live source volumes.

## Sealed Tier 2 receipt verifiers

`install/local/wikidot-verification/scripts/verify-standing-candidate-parity-admission.mjs` and `install/standing/scripts/verify-promotion-precondition.mjs` are real, side-effect-free receipt verifiers for Tier 2. They validate browser-parity evidence, candidate identity, the final-frozen receipt (a passing consolidated validation of the candidate source plus its sealed image identities; no human review or attestation input), the sealed build inventory, and the rendered staging-home binding. `install/standing/promote.mjs` chains the admission CLI, canonical promotion validator, image preparation, and standing refresh. Refresh owns activation, rollback, and failure receipts; the controller does not remove those artifacts or forward arbitrary Compose arguments. They do not build images, render or replace the canonical home, enter maintenance, run Compose, or change routing.

Use `promote.mjs` for the complete post-merge sequence. Its explicit paths are the candidate receipt, final-frozen receipt, candidate identity, live reference, live completion policy, sealed build evidence, rendered staging home, admission output, promotion precondition, prepared receipt, standing runtime home, and standing receipt. The standalone commands below remain available for receipt-only validation:

```sh
node install/local/wikidot-verification/scripts/verify-standing-candidate-parity-admission.mjs \
  --receipt /secure/candidate/standing-candidate-parity-receipt.json \
  --candidate-identity /secure/candidate/candidate-parity-identity.json \
  --live-reference /secure/live/standing-browser-live-reference.json \
  --live-completion-policy /secure/live/standing-live-completion-policy.json \
  --output /secure/candidate/standing-candidate-parity-admission.json
```

```sh
node install/standing/scripts/verify-promotion-precondition.mjs \
  --receipt /secure/candidate/standing-candidate-parity-receipt.json \
  --final-frozen-receipt /secure/candidate/final-frozen-receipt.json \
  --candidate-identity /secure/candidate/candidate-parity-identity.json \
  --live-reference /secure/live/standing-browser-live-reference.json \
  --live-completion-policy /secure/live/standing-live-completion-policy.json \
  --build-evidence /secure/build/sealed-build \
  --staging-home /secure/runtime/wikijump-standing.stage \
  --output /secure/promotion/candidate-parity-admission.json
```

The live completion policy and isolated candidate capsule remain explicit Tier 2 inputs. These verifiers are unrelated to the retired CWG01 admission harness.
