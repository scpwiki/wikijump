# Standing runtime operations

`compose.yaml` is the source-owned topology for the browser-facing standing runtime. The rendered home uses the role-based `wikijump-standing` project and retains the established external volume names `runtime50x-postgres-data`, `runtime50x-files-data`, `runtime50x-cache-data`, `local-caddy-data`, and `local-caddy-config`. Never run `docker compose down -v` against this topology.

There are two operational tiers. Routine application refreshes use Tier 1. Tier 2 is reserved for operations that can change a named volume attachment, the Compose project name, the network name, or edge routing.

## Tier 1: routine merged-head refresh

Tier 1 is the expected default after a change merges to `develop`. It rebuilds and replaces only Deepwell, Framerail, and WWS. It does not stop Caddy, database, files, or cache; it does not run `down`; and its command-line parser has no volume-removal or Compose passthrough option.

A Tier 1 refresh is required before anyone asserts that a browser-visible, chrome, layout, or DOM defect is fixed or still present in the standing runtime. A browser claim against an older standing SHA is not evidence about current code.

Start from a clean checkout whose `HEAD` equals the fetched `origin/develop` head. Run the controller under the host resource lease because its three Docker builds are shared-host heavy work:

```sh
git fetch origin develop
/home/roku/.local/bin/roku-resource-lease run exclusive -- \
  python install/standing/refresh.py \
    --source-root "$PWD" \
    --runtime-home /home/roku/wjlab/runtime/wikijump-standing
```

The controller performs one fixed sequence:

1. Verify the source checkout is clean and exactly matches `origin/develop`, then read the exact Wikijump tree and FTML pin.
2. Build Deepwell, Framerail, and WWS from `install/local/<service>/Dockerfile`; Framerail receives `--build-arg FRAMERAIL_ENV=local`.
3. Atomically update the three `STANDING_*_IMAGE` values, `STANDING_WIKIJUMP_SHA`, `STANDING_FTML_SHA`, `STANDING_LOCALES_SOURCE`, and the refresh resource expiry in the runtime `.env`. The locales bind points at the same clean source root used for the image builds.
4. Run `docker compose --project-name wikijump-standing up --detach --no-deps deepwell framerail wws` with the checked-in refresh label overlay. The overlay adds owner and expiry labels to the three recreated containers and has no volume declarations.
5. Wait for all three services to become healthy, fetch `http://scp-wiki.wikijump.localhost/scp-9506`, require the expected document markers, and overwrite `refresh-receipt.json` with the exact source, image, health, canary, and resource-disposition record.

The script refuses unknown arguments, including `-v`, `--volumes`, and `--remove-volumes`. There is no argument that is forwarded to Docker or Docker Compose.

## Rendering the canonical home

`render.py` materializes the canonical home from a clean checkout at an exact merged Wikijump revision. It copies the production Deepwell config and replaces only the production domain pair with `wikijump.localhost` and `wjfiles.localhost`; it emits the absolute source-root locales path used by the read-only `/opt/locales` bind. The renderer fails closed if the production domain block changes, the requested FTML revision is absent, either required Deepwell runtime source is missing, or the checkout identity is not exact. `identity.json` records the source tree, FTML pin, config hash, image inputs, and persistent volume names.

Rendering does not mutate the running stack. A routine Tier 1 refresh uses the already materialized home; a topology change uses Tier 2.

## Tier 2: topology, volume, network, or edge maintenance

Tier 2 is the controlled maintenance ceremony formerly described as retiring the `runtime50x` project. Use it only when an operation can touch a named volume attachment, the Compose project name, the network name, or edge routing. Routine source updates do not use Tier 2.

1. Record the current container identities, image digests, mounted volumes, network, edge owner, and passing canaries as rollback evidence. Verify the five named persistent volumes and their consumers before changing anything.
2. Build the exact merged-head images under the required lease, render a staging home bound to those identities, and validate it before touching the canonical home.
3. Activate the bounded explicit 503 maintenance response. Connection refusal, a generic 5xx, or silently serving an old candidate is not an acceptable maintenance state.
4. Stop the old standing containers without `-v`, start the reviewed topology, and verify that every named volume survives. Never prune or recreate a named data volume as part of this ceremony.
5. Verify one port-443 owner, expected project and network identities, service health, HTTP and assets, WIKIREQUEST metadata, AJAX ListPages, DOM, and unmodified `wikidot.py` lookup canaries.
6. On failure, restore the saved topology without `-v`, restore normal edge routing, and seal a failed receipt. On success, seal the promotion receipt, retain the immediate rollback image, and remove only superseded images whose lifecycle closure is proven.

Physical volume renames remain a separate data migration with rollback retention and evidence. They are never an incidental effect of Compose recreation.

## Sealed Tier 2 receipt verifiers

`install/local/wikidot-verification/scripts/verify-standing-candidate-parity-admission.mjs` and `install/standing/scripts/verify-promotion-precondition.mjs` are real, side-effect-free receipt verifiers for Tier 2. They validate browser-parity evidence, candidate identity, the sealed build inventory, and the rendered staging-home binding. They do not build images, render or replace the canonical home, enter maintenance, run Compose, or change routing.

No checked-in host mutation controller currently chains those verifiers into a deploy. Treat the commands below as sealed-receipt validation tools, not as an operational orchestrator:

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
  --candidate-identity /secure/candidate/candidate-parity-identity.json \
  --live-reference /secure/live/standing-browser-live-reference.json \
  --live-completion-policy /secure/live/standing-live-completion-policy.json \
  --build-evidence /secure/build/sealed-build \
  --staging-home /secure/runtime/wikijump-standing.stage \
  --output /secure/promotion/candidate-parity-admission.json
```

The live completion policy and isolated candidate capsule remain explicit Tier 2 inputs. These verifiers are unrelated to the retired CWG01 admission harness.
