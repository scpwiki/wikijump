# Standing runtime configuration

`compose.yaml` is the single declarative topology for the browser-facing standing runtime. It has no build directives, no candidate ports, and no references to a worktree, evidence directory, or `/tmp` override. Its persistent data volumes retain their established names so the role-based Compose project can replace the retired `runtime50x` project without a volume migration.

Render this directory only from a clean checkout at the exact merged Wikijump revision. `render.py` copies the exact Deepwell config and seeder inputs into a pinned host configuration home, writes the Compose environment and identity manifest, and refuses a dirty or revision-mismatched source checkout. The resulting `identity.json` is evidence, while the source commit, FTML pin, image identities, and promotion receipt remain the durable reconstruction inputs.

The initial materialization does not change a running stack. The later promotion procedure uses the rendered directory with the role-based project name, enters the explicit 503 maintenance protocol, brings the old stack down without `-v`, starts this topology, verifies the named persistent volumes, runs all standing canaries, and seals a promotion receipt before removing superseded candidate images.

Do not run `docker compose down -v` against this topology. Do not add candidate image overrides or arbitrary host paths to the rendered home.
