# WJ-OPS-010 dispatcher proof

This proof covers the non-destructive `wj-codex-grid` dispatcher contract at the repo-visible helper layer.

Validation command:

```bash
cd install/local/wikidot-verification
npm test
```

The `tmux-dispatcher.test.mjs` suite asserts:

- the human attach path remains `tmux attach -t wj-codex-grid`;
- destructive reset requires explicit `confirmReset=true` and is not used by enqueue/dispatch;
- enqueue dispatch writes generation-scoped assignment files and emits `TMUX_DISPATCHED` events;
- normal dispatch uses `tmux send-keys` to a stable `wj-codex-grid:0.0` target and never sends `kill-session`;
- two sequential assignments in the same lane reuse the same pane target and end in `DONE_REUSABLE` with `ARTIFACT_SCHEMA_PASS` events through the existing PR #48 helper semantics.

The included `wj-grid-dispatch.mjs` entrypoint is the non-destructive enqueue/attach/reset CLI. The included `wj-grid-worker-once.mjs` entrypoint runs one assignment from a lane using the existing artifact validation and `DONE_REUSABLE` state machine.
