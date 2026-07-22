# Local Wikidot verification tools

The scripts in this directory import frozen Wikidot corpus data, inspect a local runtime, capture browser evidence, and reduce large runs into machine-readable verdicts. Expected behavior must come from the frozen corpus, reviewed compatibility policy, or sealed real-Wikidot evidence. Local Wikijump output is diagnostic evidence, not an oracle.

## Python environment

The authenticated Wikidot helper runs from this component's private `.venv`, with the owner's `Rokurolize/wikidot.py` fork pinned by commit in `requirements.txt`. Create or refresh the environment before using the theme-localization execution path:

```sh
install/local/wikidot-verification/scripts/setup-python-env.sh
```

The helper never imports from a mutable host checkout. Credentials remain environment-only inputs to the helper process.

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

`scripts/capture-browser-rendering.mjs` uses a fixed host-wide capture lock and durable request-gate state under `/var/tmp/`. Every non-local HTTP(S) browser request in either source or local context is admitted at no more than one request per four seconds, including documents, redirects, frames, scripts, stylesheets, images, and fetches. The gate persists its next admissible time and any observed `Retry-After` deadline before a request proceeds, so a later capture cannot reset the rate. Service workers and WebSockets are blocked. The command accepts only canonical standing `https://<site>.wikijump.localhost` page URLs as local exemptions and derives the matching `https://<site>.wjfiles.localhost` file origin; public or credentialed inventory values fail before browser startup. It seals `request-gate-config.json` before starting the proxy or browser and records final gate counters in `records.json`. A failed state confirmation leaves the lock pending and blocks a later capture until an operator reviews it.

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

`scripts/run-standing-browser-parity.mjs` defines the source-owned browser-parity receipt that an explicit promotion-controller migration will make the standing promotion precondition. The current host controller already blocks on its existing candidate-parity receipt before mutable standing operations, but it has not yet been migrated to this source-owned verifier. The runner has two intentional modes. `live-reference` captures only the six production-theme canaries from `scp-wiki.wikidot.com` through the existing persistent 0.25 req/s gate. `candidate` captures the same pages only from a sealed, expiring non-443 candidate and compares them to an exact sealed live reference. Neither mode targets port 443.

```sh
node install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs \
  --mode live-reference \
  --output-dir /mnt/oracle-store/wjlab/standing-live-reference-... \
  --live-completion-policy /secure/standing-live-completion-policy.json \
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

The live policy is sealed before any browser request and names each tolerated external failure exactly. A candidate identity is sealed before local capture and binds its repository/tree, FTML pin, immutable image IDs, isolated configuration hashes, owner/expiry, non-443 endpoint, loopback address, and evidence seal. Before opening the browser, and again after browser, proxy, request-gate, and lock closure, the runner independently inspects the candidate Compose project. Every declared role must be running exactly once with the sealed image, provenance labels, expiry, artifact key, configuration hash, and a Caddy HTTPS mapping limited to the declared loopback non-443 endpoint. The identity also carries the expected aggregate hash of effective Docker service configuration. The runner computes that hash from command, entrypoint, environment, mounts, network, port, and security settings without recording secret values. Candidate mode runs only from a clean source checkout whose exact Wikijump tree and FTML lock pin match the candidate, and binds a canonical manifest of every parity module into the receipt. The resulting receipt rejects a missing canary, stale candidate, mutable image tag, altered runtime configuration, runtime replacement during capture, local-only anomaly, omitted screenshot, incomplete load/font/image observation, or a record that lacks the `DOMContentLoaded` observation. A terminal ledger and receipt are published only after clean closure, with the final shared-gate snapshot bound into both. The immediate capture is DOM/CSS evidence at `DOMContentLoaded`, not a compositor-filmstrip claim.

For a direct Framerail candidate that is not behind WWS, `--site-id ID` injects the non-secret trusted routing identity for the fixed `scp-wiki` authority. Omit it when exercising the complete edge path.
