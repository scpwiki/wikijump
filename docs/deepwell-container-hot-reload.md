# Deepwell container hot reload

`install/local/deepwell_hot_reload.py` is the short iteration path for a
prebuilt local runtime such as the corpus `runtime50x` stack. It copies only
Deepwell's Rust build inputs into the already-running container. The image's
existing `cargo-watch` process recompiles and restarts Deepwell without a cold
Docker image build.

This complements the normal `docker-compose.dev.yaml` workflow. The normal
development stack bind-mounts sources and already reloads them automatically;
the helper deliberately refuses to write across those mounts.

## Typical runtime50x iteration

Run the helper from the worktree whose candidate should be tested:

```bash
./install/local/deepwell_hot_reload.py --project runtime50x
```

Container discovery uses the standard Compose project and service labels, so
the command does not depend on the location or name of a lab-specific Compose
override. An explicit container and source worktree can be supplied when
needed:

```bash
./install/local/deepwell_hot_reload.py \
  --container runtime50x-deepwell-1 \
  --source-root /home/user/src/wikijump
```

Use a dry run before targeting an unfamiliar stack:

```bash
./install/local/deepwell_hot_reload.py \
  --project runtime50x \
  --dry-run \
  --json
```

The normal successful path performs these checks and actions:

1. Verify the source is a Deepwell worktree and resolve one running container.
2. Require a healthy Deepwell daemon, a writable `/src/deepwell`, and a live
   `cargo-watch` process.
3. Refuse any container mount overlapping the copied inputs. This prevents a
   `docker cp` from unexpectedly changing bind-mounted host files.
4. Copy `src`, `Cargo.toml`, `Cargo.lock`, `build.rs`, and `askama.toml` to a
   unique staging directory inside the container.
5. Replace the staged inputs as one debounced change set. Complete directory
   replacement means deleted Rust files do not remain stale in the container.
6. Wait for the old Deepwell PID to be replaced, then require the new PID to
   remain stable and pass the container's JSON-RPC health check.
7. Detect an unreaped old Deepwell daemon. Some prebuilt stacks run
   `cargo-watch` directly as PID 1, so no init process adopts and reaps the old
   daemon. When this occurs, restart the container once, retain the copied
   inputs and build cache, and require a stable healthy daemon with zero
   remaining Deepwell zombies.

On a compile or health failure the helper prints recent container logs and
restores the previous build inputs. It also restores them when interrupted
during a synchronous candidate build. Both paths then restart the Deepwell
container to terminate any superseded candidate build or daemon, require the
recovered PID to remain stable and healthy, and return nonzero. If rollback
itself fails, the error names the staging path and leaves its backup in the
container for recovery. `--no-wait` is intentionally asynchronous and
therefore cannot provide this verified rollback or zombie-cleanup guarantee.
Concurrent helper
runs against one container from the same host are rejected by an OS file lock
that is automatically released even if the helper is interrupted. The daemon
identity comes from Deepwell's PID file, so short-lived `render-replay` workers
cannot be mistaken for the server restarted by `cargo-watch`.

## When a cold rebuild is still required

Rebuild the image when changing the Rust toolchain, system packages, the
Deepwell Dockerfile, or container startup scripts. The hot-reload helper does
not copy secrets, runtime configuration, locales, migrations, seed data, or
other services. Apply schema migrations through the normal migration workflow;
the initial container startup migration step is outside `cargo-watch`.
