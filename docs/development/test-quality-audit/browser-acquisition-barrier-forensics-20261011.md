# Offline browser-acquisition barrier forensics (2026-10-11)

Tracking: #1990, #2325. A 2026-10-11 attempt to compare actual `scp-wiki.wikidot.com/theme:yossistyle` against the local mirror wrote two HTTP-200 DOM artifacts and paired Chromium screenshots, but **failed admission**: the request gate snapshot said `enforcement_failed: true`; the capture/cleanup exception was an `AggregateError` and the capture lock was left unconfirmed. The previously merged #2337 fixes loss of nested exception messages in the CLI for **future authorized** attempts, but cannot retroactively recover this attempt's original exceptions. The captured HTML and images must **not** be promoted to a trustworthy browser oracle or a parity PASS.

## Non-mutating inspection

Use `inspect-browser-capture-barriers.mjs` on existing evidence only; it has no network, browser, cache writer, lock release or retry implementation. The output never prints a request path, query string, cache body, account name, or credential. When a lock is provided, its run ID, host, PID and process-start ticks must match the independently retained request-gate configuration. A mismatch fails without attributing the lock to the run.

```
node install/local/wikidot-verification/scripts/inspect-browser-capture-barriers.mjs \
  --records /tmp/wj-1990-yossistyle-current-live-20261011/records.json \
  --cache-manifest "$HOME/.cache/wikijump-verification/candidate-public-evidence-v1/manifest.json" \
  --lock /var/tmp/wikijump-wikidot-browser-capture.lock \
  --request-gate-config /tmp/wj-1990-yossistyle-current-live-20261011/request-gate-config.json
```

The inspected input identities were:

- Captured `records.json`: SHA-256 `fec3708c9adf23f70445dd6353d44cb75b5eb06c3a001196371c64ecc490db73`
- Capture request-gate config: SHA-256 `139d08a2ed5b111d485393353485cb42b386363368e902793f10612152f4dcac`
- Persistent cache manifest as inspected: SHA-256 `ce55dd54ea758149f6404c68fd694903b82833ac1ea3e3e3114e894f4f00fc86`
- Unconfirmed lock as inspected: SHA-256 `5307010ab766f3cb7d1c022ff187a705e61b6e0f9796d5f88509baee416db38f`

Do not check the raw cache manifest, cookies, captured HTML, or personal data into the repository. All of these exact paths are environment-specific; absent inputs should fail closed rather than fabricate evidence.

Read-only result: the source reported **9 failed browser resource requests**, of which **3 exact URLs** matched the *subsequently inspected* cache acquisition barrier keys. Their hosts were `ab.dns-finder.com`, `btloader.com`, and `cdn.confiant-integrations.net` (external advertising/tracking dependencies). At the capture's snapshot, there were **15 acquisition barriers**; the later inspected shared cache has **17**. The counts differ, hence the later persistent manifest cannot be treated as a contemporaneous inventory or used to assign every barrier to this one capture. The exact cause(s) of the gate's original failure therefore remain **UNVERIFIED**. The source and local document status were both HTTP 200; that does not override fail-closed admission.

This investigation left `/var/tmp/wikijump-wikidot-browser-capture.lock` in its original `state_confirmation: pending` state, preserved all persistent cache acquisition barriers, performed **no source reacquisition**, and did not touch standing or 443. Operator review is required before any attempted cleanup/recovery; deleting the lock or using a new cache identity to bypass barred requests would destroy the audit guarantee.

Unit tests cover request/cache identity mismatch, lock-to-run binding, refusal of malformed URL schemes/credentials, no sensitive URL paths or queries in report output, absent barrier inventory, and temporal count divergence. The script is a forensic tool, not a source-compatibility or full-page-visual parity gate.
