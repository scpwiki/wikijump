# ListPages scanner — reproducible mutation ledger (2026-10-10)

Immutable, portable copies of **33 SHA-256-sealed cargo-mutants outcome files** used to reconcile 136 distinct mutation IDs: 8 initial shards and 25 targeted replays. `manifest.json` records their order, roles and exact SHA-256 digests. Outcome bytes are preserved from independently verified mutation-run files, without reliance on the original `/tmp` locations.

From the repository root run:

```sh
node docs/development/test-quality-audit/scanner-mutation-ledger-20261010/verify.mjs
```

The verifier checks every receipt SHA, regenerates the reconciliation using the repository's maintained script, independently re-verifies the generated report, and checks the exact aggregate and remaining survivor list. The temporary report is removed afterward.

**Expected: 132 Caught / 0 Missed / 4 Unviable / 0 Timeout (136 unique scanner mutation IDs).** The four `Unviable` entries remain build-classification results, **not semantic-equivalence approvals**. Zero missed mutations in this frozen *scanner-only* campaign **does not certify Wikidot behavior**, all production files or Issue #1990 acceptance, and does not authorize standing/443 promotion.

Most recent fully checked replays: `scanner.rs:1913:25 && -> ||` (Caught); `1912:24 delete !` (Caught after avoiding previous infrastructure failures); and `1645:21 && -> ||` (Caught with the **already committed** `original_css_ownership_filters_projected_structural_events` regression, not the parallel Codex agent's uncommitted test). This last test is an internal CSS/anchor ownership guard, **not a newly collected independent live Wikidot oracle**. External behavioral validation and the repository-wide production-file audit remain outstanding.

The successful deletion-`!` replay used `CARGO_BUILD_JOBS=1`, `-C --lib`, and a sparse checkout that included the Caddyfile fixtures required for compiling lib tests. Attempts that failed due to SIGTERM, ENOSPC, or missing fixture files were **excluded** from this complete mutation ledger.
