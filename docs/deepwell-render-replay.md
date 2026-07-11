# Deepwell render replay

`render-replay` is the read-only convergence path for corpus render failures. It
loads each page through the same revision, score, layout, include, module,
protection, and FTML preprocessing code as production, then tokenizes, parses,
and core-renders it in a separate process. A worker that exceeds its deadline is
force-killed and reaped; it cannot leave a runaway parser thread in Deepwell.

Inside the `runtime50x` development container, after the hot-reload build has
finished, run:

```sh
docker exec \
  -e DEEPWELL_RUNTIME_ACTION=render-replay \
  -e DEEPWELL_REPLAY_IMPORT_RUN_ID=217 \
  -e DEEPWELL_REPLAY_ARTIFACT_DIR=/tmp/run217-render-replay \
  runtime50x-deepwell-1 \
  /src/deepwell/target/debug/deepwell /etc/deepwell.toml
```

For a host-side release build, supply the host's normal Deepwell secrets and
configuration path to `cargo run --release -- <config-path>` with the same
`DEEPWELL_RUNTIME_ACTION` and `DEEPWELL_REPLAY_*` variables.

When `DEEPWELL_REPLAY_IMPORT_RUN_ID` is omitted, the newest import run with a
matching item is selected. The defaults are:

- `DEEPWELL_REPLAY_STATES=render_failed`; every matching row is replayed (there
  is no hidden batch limit).
- `DEEPWELL_REPLAY_CONCURRENCY=8`; values from 1 through 16 are accepted.
- `DEEPWELL_REPLAY_TIMEOUT_MS=10000` per isolated worker.
- `DEEPWELL_REPLAY_DDMIN=true` and
  `DEEPWELL_REPLAY_DDMIN_MAX_PROBES=128` per failure cluster.
- A process-unique directory under `/tmp` when no artifact directory is given.

An explicitly selected artifact directory must not exist, and its parent must
already exist. Deepwell creates the leaf atomically with owner-only
permissions; every fixed artifact name is also created without replacement.
Parents or ancestors writable by other users are accepted only when the
sticky bit protects their entries, as it does for a normal `/tmp`.
This fail-closed rule prevents stale evidence and pre-created symlinks from
being mistaken for, or overwritten by, the current run.

The action prints one `deepwell.render-replay.v1` JSON summary and writes the
same summary to `summary.json`. The directory also contains:

The summary's `gate_passed` field is true, and the process exits successfully,
only when an explicit or selected import run yields a non-empty candidate set,
every selected candidate passes through FTML without compatibility fallback,
and every minimization is verified. Otherwise `gate_failures` lists stable
machine-readable reasons and the process exits nonzero. This replay-local gate
does not by itself certify that the selected states cover the full manifest.

- `observations.jsonl`, with outcome, stable failure signature, syntax
  features, error positions, stage timings, and hashes per page;
- `page-<id>.capsule.json` and `page-<id>.expanded.wikidot`;
- `page-<id>.preprocessed.wikidot`, atomically persisted by the worker before
  tokenization so it survives a parser timeout;
- one directory per deterministic failure cluster, containing
  `min.expanded.wikidot` and, when preprocessing completes,
  `min.preprocessed.wikidot`.

Case artifacts and JSONL observations are persisted as each bounded worker
finishes. Expanded page bodies are then released; ddmin reloads only the next
cluster representative from its capsule. A full replay therefore does not
retain every expanded corpus page in memory.

Parser-error ddmin probes use the requested replay concurrency. Timeout probes
run one at a time because parallel wall-clock probes contend for CPU and can
turn a fast candidate into a false timeout reproducer. Every minimized result
is rerun once in isolation. The summary records its probe concurrency, final
outcome and fingerprint, and `verified`; a timing-sensitive mismatch remains
explicit evidence instead of aborting and discarding the complete replay
summary.

The `ftml_core_rendered_sha256` field hashes FTML's core HTML only. It is a
diagnostic fingerprint, not proof of Deepwell post-render compatibility. After
fixing a cluster, run the affected replay set and then perform one normal
runtime rerender/browser validation.

Production rendering uses random nonces for unforgeable protection markers.
Replay canonicalizes only those known nonce-bearing markers to stable,
same-length ordinal tokens before writing preprocessed artifacts, error
contexts, and diagnostic hashes. The core-HTML hash likewise canonicalizes
FTML's random `wj-id-*` element IDs and their ARIA references. Replaying an
unchanged capsule therefore produces comparable evidence without weakening the
production trust boundary.

Replay opens read transactions and rolls them back. It does not claim import
items, update render state, write compiled HTML, or start background job
workers. Isolated parser workers receive only their serialized replay capsule;
the controller clears database, object-store, and other environment secrets
before spawning them.

The frozen corpus snapshot stores net page rating but older rows do not store
the ListPages `%%rating_votes%%` count. Deepwell reads `votes_count` from the
snapshot metadata when present. When absent, it supplies zero so components
select their explicit no-votes state; it never invents an upvote/downvote count
from the net score. `[[#expr]]` and `[[#ifexpr]]` use a bounded 256-byte grammar
covering arithmetic, comparisons, boolean literals, `&&`/`||`, and
`abs`/`min`/`max`; the false branch of `[[#ifexpr]]` is optional. Division by
zero produces numeric zero only so a hidden nonzero-vote branch cannot leak
parser syntax. Malformed or unknown expressions remain literal and therefore
remain replay failures.
