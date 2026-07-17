# Local Wikidot verification tools

The scripts in this directory import frozen Wikidot corpus data, inspect a local runtime, capture browser evidence, and reduce large runs into machine-readable verdicts. Expected behavior must come from the frozen corpus, reviewed compatibility policy, or sealed real-Wikidot evidence. Local Wikijump output is diagnostic evidence, not an oracle.

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
