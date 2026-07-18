# Standing runtime configuration

`compose.yaml` is the single declarative topology for the browser-facing standing runtime. It has no build directives, no candidate ports, and no references to a worktree, evidence directory, or `/tmp` override. Its persistent data volumes retain their established names so the role-based Compose project can replace the retired `runtime50x` project without a volume migration.

Render this directory only from a clean checkout at the exact merged Wikijump revision. `render.py` copies the exact Deepwell config and seeder inputs into a pinned host configuration home, writes the Compose environment and identity manifest, and refuses a dirty or revision-mismatched source checkout. The resulting `identity.json` is evidence, while the source commit, FTML pin, image identities, and promotion receipt remain the durable reconstruction inputs.

The initial materialization does not change a running stack. The later promotion procedure uses the rendered directory with the role-based project name, enters the explicit 503 maintenance protocol, brings the old stack down without `-v`, starts this topology, verifies the named persistent volumes, runs all standing canaries, and seals a promotion receipt before removing superseded candidate images.

## Retiring the `runtime50x` project

The one-time role-name promotion is a controlled maintenance operation, not a Compose cleanup shortcut. It may start only after the canonical-config PR is merged, the rendered home is bound to an exact merged source and image identity, and a read-only receipt proves which volumes contain the production corpus, which live containers reference them, and which legacy volumes must remain archived.

1. Preserve the current container identities, image digests, mounted volumes, and passing canaries as the rollback record. Confirm that `runtime50x-postgres-data`, `runtime50x-files-data`, `runtime50x-cache-data`, `local-caddy-data`, and `local-caddy-config` are still present and that no target legacy or candidate container is using them unexpectedly.
2. Build the Deepwell, Framerail, and WWS images from the exact merged head under the appropriate exclusive build lease. Record their digests and FTML pin, then render the canonical host home with `STANDING_PROJECT_NAME=wikijump-standing` and `STANDING_NETWORK_NAME=wikijump-standing_default`.
3. Activate the bounded explicit 503 maintenance response on the current browser edge. A connection refusal, a generic 5xx, or silently serving an old candidate is not an acceptable maintenance state.
4. Stop the old standing containers without `-v`, start the canonical Compose home as `wikijump-standing`, and wait for every service health check. Do not run any broad Docker prune or remove a named volume during this transition.
5. Confirm that the new containers mount the exact named production volumes, that port 443 has exactly one `wikijump-standing` owner, and that no `runtime50x` container remains browser-facing. Run HTTP and asset, WIKIREQUEST metadata, AJAX ListPages, DOM, and unmodified `wikidot.py` lookup canaries.
6. If health or any canary fails, stop the new project without `-v`, restore the saved old containers, restore normal browser routing, and seal a failed-promotion receipt. If every check passes, seal the promotion receipt, retain the immediate rollback image, and remove only superseded candidate images that have completed their image-lifecycle closure proof.

The production data volumes deliberately keep their current names. Renaming them is an optional future data migration, never an incidental effect of the Compose project rename.

Do not run `docker compose down -v` against this topology. Do not add candidate image overrides or arbitrary host paths to the rendered home.
