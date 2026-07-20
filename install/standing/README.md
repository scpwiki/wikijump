# Standing runtime configuration

`compose.yaml` is the single declarative topology for the browser-facing standing runtime. It has no build directives, no candidate ports, and no references to a worktree, evidence directory, or `/tmp` override. Its persistent data volumes retain their established names so the role-based Compose project can replace the retired `runtime50x` project without a volume migration. Its rendered Caddy request explicitly uses local certificates, preserving the Caddy image health check's TLS probe without trying public ACME issuance for local standing domains.

Render this directory only from a clean checkout at the exact merged Wikijump revision. `render.py` copies the production Deepwell config into a pinned host configuration home and replaces only its production domain pair with the standing runtime's `wikijump.localhost` and `wjfiles.localhost` domains. The replacement fails closed if the production domain block changes, while all production security, seeding, timeout, and mail settings remain unchanged. The renderer writes the source-config hash and domain override into the identity manifest and refuses a dirty or revision-mismatched source checkout. The resulting `identity.json` is evidence, while the source commit, FTML pin, image identities, and promotion receipt remain the durable reconstruction inputs.

The initial materialization does not change a running stack. The later promotion procedure uses the rendered directory with the role-based project name, enters the explicit 503 maintenance protocol, brings the old stack down without `-v`, starts this topology, verifies the named persistent volumes, runs all standing canaries, and seals a promotion receipt before removing superseded candidate images.

## Candidate browser parity before promotion

Every ordinary standing promotion stops before it can materialize the canonical home, enter maintenance, or recreate a backend unless it has a sealed `wikijump.standing_candidate_parity_receipt.v1`. The current host controller already enforces that candidate-receipt boundary with its host-owned validator. `install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs` defines the reviewed source-owned successor receipt contract; landing this source contract does not silently replace the controller's active validator.

The source-owned canary contract covers six EN pages across Sigma, Basalt, Flopstyle Y2K, and Black Highlighter Calibri. `scp-9506` probes the header logo, title, subtitle, navigation tab bar, and tab links. `theme:basalt` probes `.yui-navset`. For every canary, the runner records a viewport screenshot, theme custom properties, selector probes, and generated-content geometry immediately after `DOMContentLoaded`; it then records settled viewport and full-page screenshots. The immediate observation is deliberately not called compositor first paint: it detects a late theme/style state relative to DOM readiness but does not claim a filmstrip-level guarantee.

First seal a read-only live reference using an owner-approved completion policy. The policy starts the shared gate at 0.25 requests per second, honors `Retry-After`, and names each tolerated external failure exactly. Then measure a separately owned, expiring, non-443 candidate whose sealed identity names its source commit/tree, FTML pin, immutable image IDs, configuration hashes, endpoint, and local connect address. Its isolated Compose overlay must stamp every candidate container with the sealed owner, expiry, Wikijump commit/tree, FTML SHA, artifact key, isolated-overlay SHA, effective-service-configuration SHA, profile, and role labels. Candidate-local origins are exempt from the public request gate; all other HTTP(S) requests remain metered. The runner verifies those labels, exact immutable image IDs, exactly one healthy running container per declared role, the effective configuration hash over command, environment, mounts, network, ports, and security settings, and the exact loopback Caddy publication before and after capture. It rejects a mutable image tag, a port-443 endpoint, incomplete canary coverage, runtime replacement, a missing screenshot, incomplete load/font/image observation, a post-settle-only record, an unbound live reference, and any passing comparison with an anomaly or omitted required probe. It seals a terminal receipt only after browser, proxy, gate, and lock closure has succeeded.

The source-owned receipt integrity verifier below has no runtime side effects. It rehashes the candidate ledger and every local screenshot, reloads the live reference under the sealed completion policy, checks the candidate's clean source-tree/module manifest, and rejects a receipt produced by another runner. `install/standing/scripts/verify-promotion-precondition.mjs` adds the promotion-specific binding check: it verifies the sealed build's exact manifest and seven-image inventory, compares the accepted candidate to that build and the rendered staging-home manifest, then writes a no-replace admission receipt. It has no Docker, maintenance, canonical-home, or network side effects.

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

The host mutation controller must first run its complete sealed-build validator, then invoke the second command after it renders and validates the staging home, and before it materializes the canonical home, enters maintenance, or recreates a backend. The adapter validates candidate-binding inputs and does not replace the controller's broader build-provenance checks. The controller may accept only a passing, sealed output from this exact source adapter. Wiring that operational call is a separately receipted host change; until it is installed, the existing host-owned validator remains the active production boundary.

The live completion policy and canonical isolated data/config capsule remain explicit operational inputs. Do not invent either in source code or use the standing runtime as a candidate. This no-side-effect receipt verifier is distinct from CWG01's paused performance admission harness and must not be used to revive CWG01, its containers, or that harness.

## Retiring the `runtime50x` project

The one-time role-name promotion is a controlled maintenance operation, not a Compose cleanup shortcut. It may start only after the canonical-config PR is merged, the rendered home is bound to an exact merged source and image identity, and a read-only receipt proves which volumes contain the production corpus, which live containers reference them, and which legacy volumes must remain archived.

1. Preserve the current container identities, image digests, mounted volumes, and passing canaries as the rollback record. Confirm that `runtime50x-postgres-data`, `runtime50x-files-data`, `runtime50x-cache-data`, `local-caddy-data`, and `local-caddy-config` are still present and that no target legacy or candidate container is using them unexpectedly.
2. Build the Deepwell, Framerail, and WWS images from the exact merged head under the appropriate exclusive build lease. Build the Framerail standing image with `--build-arg FRAMERAIL_ENV=local` only so its SvelteKit CSP is compiled for the local runtime; keep the production Dockerfile default `FRAMERAIL_CSRF_CHECK_ORIGIN=true` in place so SvelteKit still rejects cross-origin form submissions. Record their digests and FTML pin, then render the canonical host home with `STANDING_PROJECT_NAME=wikijump-standing` and `STANDING_NETWORK_NAME=wikijump-standing_default`.
3. Activate the bounded explicit 503 maintenance response on the current browser edge. A connection refusal, a generic 5xx, or silently serving an old candidate is not an acceptable maintenance state.
4. Stop the old standing containers without `-v`, start the canonical Compose home as `wikijump-standing`, and wait for every service health check. Do not run any broad Docker prune or remove a named volume during this transition.
5. Confirm that the new containers mount the exact named production volumes, that port 443 has exactly one `wikijump-standing` owner, and that no `runtime50x` container remains browser-facing. Run HTTP and asset, WIKIREQUEST metadata, AJAX ListPages, DOM, and unmodified `wikidot.py` lookup canaries.
6. If health or any canary fails, stop the new project without `-v`, restore the saved old containers, restore normal browser routing, and seal a failed-promotion receipt. If every check passes, seal the promotion receipt, retain the immediate rollback image, and remove only superseded candidate images that have completed their image-lifecycle closure proof.

The production data volumes deliberately keep their current names. Renaming them is an optional future data migration, never an incidental effect of the Compose project rename.

Do not run `docker compose down -v` against this topology. Do not add candidate image overrides or arbitrary host paths to the rendered home.
