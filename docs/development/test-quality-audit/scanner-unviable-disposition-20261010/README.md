# Four scanner Unviable function-replacement mutations — compiler disposition

Scope: the frozen **136-ID `find_list_pages_module_matches_with_cursor_work_context_lowercase`** mutation cohort in [`../scanner-mutation-ledger-20261010`](../scanner-mutation-ledger-20261010). The frozen outcome is **132 Caught, 0 Missed, 4 Unviable, 0 Timeout**. This report does not extend that cohort to other scanner functions or other production code.

All four `Unviable` outcomes originate at `scanner.rs:1426:5` and replace **the entire scanner function** with return values `(vec![Default::default()], 0, 0)`, `(vec![Default::default()], 0, 1)`, `(vec![Default::default()], 1, 0)`, or `(vec![Default::default()], 1, 1)`. Each generated function-body replacement fails at Rust compile time with **E0277**: `ListPagesModuleMatch<'_>` does not implement `Default`. The return type is `Vec<ListPagesModuleMatch<'a>>` and the original struct derives `Debug` but not `Default` (nor does it provide a `Default` implementation). This failure occurs **before** any behavioral test can run; adding assertions cannot make these exact generated replacements type-check.

The `compiler-logs/` files are **deterministic gzip archives containing byte-for-byte the four complete historical compiler logs** from the initial cargo-mutants shard. Both compressed and decompressed SHA-256 values, alongside precise mutation IDs are pinned in `manifest.json`. This is not an inferred compiler error or a self-oracle from the scanner output. Run from the repository root:

```sh
node docs/development/test-quality-audit/scanner-unviable-disposition-20261010/verify.mjs
node docs/development/test-quality-audit/scanner-mutation-ledger-20261010/verify.mjs
```

The first command verifies the compressed and decompressed compiler log bytes, mutation identity, Rust E0277 diagnostic and linkage to the frozen portable ledger. The second independently recomputes the outcome totals from all 33 SHA-fixed receipts.

**Disposition:** these four exact generated replacements are **structurally non-compilable mutants**, so do not relabel them as Caught, Missed, Timeout or semantically equivalent. No alternate implementation variant was examined. The frozen scanner run has zero *viable missed* mutants, **not** final-zero for Wikijump. The CSS/anchor ownership test for `1645:21` remains an internal scanner invariant; independent Wikidot/DOM behavioral parity, broader high-value owners and the quality of individual assertions remain subject to Issue #1990. No standing/443 promotion and no Issue closure is justified by this report alone.
