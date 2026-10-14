# Candidate and standing runtime

## Build and freeze the material identity

Develop in the local incremental stack; produce production/release images only after source batch is review-ready and release suffix is needed. Commit and push the exact tree before the immutable build so review, build, and measurement share one identity. Freeze generated denominators, reviews, fixtures, source-owned verification tools, source-writer roster, and image-producer inputs from that tree; regenerate source-bound final-frozen inputs when source identity changes, preserving exact frozen verifier/tool bytes or their source-owned snapshot when a receipt binds them.

## Candidate inputs and execution

Use maintained producer/start scripts, their Compose naming/topology, and ownership contract; `private-runtime.json` is runtime plumbing, not a candidate case input. Generate candidate-private fixtures through the maintained compatibility input producer so TLS, actor/session, temporal, and canary assets bind to the exact candidate. Treat the generated candidate CaseSet manifest as the denominator: counts and aliases may change, so run every executable CaseSet, retain per-case receipts and cleanup results, and aggregate against that manifest and candidate identity. Use persistent identity-bound external replay: local candidate/WSL services need no spacing, while public origins should normally receive zero repeat requests after the first retained acquisition. Bind browser candidate parity to the accepted candidate identity, retained live reference, live-completion policy, exact browser executable identity when required, and identity-bound response cache; diagnose browser/cache/route asymmetry before calling a resource anomaly a renderer regression.

## Promotion and merge

Generate and retain the promotion precondition while PR head is candidate source. After merge, `install/standing/prepare.py` revalidates the retained candidate/precondition against normal merge; never regenerate admission from the merge checkout, whose commit is not the candidate. Use normal two-parent merge and `docs/deployment/runtime-drift-policy.md`; let `install/standing/merge_identity.py` / `install/standing/prepare.py` decide eligibility from actual Git delta and FTML identity, never filenames. Reuse a policy-approved sealed image only when eligible; a rejected runtime-input delta requires a new sealed candidate.

## Saved-page freshness

Persisted render semantics are standing identity: expected `compiled_generator` derives from the current FTML version and Deepwell renderer epoch; increment the epoch when persisted HTML semantics change so the stale-page gate distinguishes old artifacts. Fail-closed refresh requires active latest saved pages current before acceptance. Corpus-backed pages use the source-owned trusted render finalizer, not direct compiled-HTML writes; non-corpus/tail pages use source-owned rerender or a transient queue invoking the normal renderer, whose state is removed after compiled-artifact verification. For a generation bump, render ahead with `install/standing/render_ahead.py` before activation (see `install/standing/README.md`): the new build renders into `page_revision_render_ahead` while the old one serves and promotes valid rows at startup, so no maintenance window is needed for the rerender. `stale=0` is necessary but insufficient: persistent source, user, site, navigation, file, or other runtime state can differ from the accepted candidate, so fresh standing browser/full-page proof must catch it.

## Standing activation and proof

Activate only through maintained `install/standing/prepare.py`/`install/standing/refresh.py`; preserve protected standing volumes. The refresh receipt's saved-page freshness section must PASS with zero stale pages; a validator rejecting/ignoring new freshness fields is a verifier/schema defect, not grounds to drop them. After activation, fresh standing evidence must bind merge identity; candidate PASS is never standing PASS. Use the maintained comparison contract, retained reference's exact browser/cache identity, zero-repeat external replay, and full-page/canary checks. If fresh standing parity fails after a healthy promotion, compare active source hashes, `compiled_generator` freshness, site/user/file metadata, browser/resource failures, and exact comparison contract before mutating; repair the smallest authoritative persistent state and never copy an entire candidate database into standing to make a canary pass.

## Runtime cleanup

Retain active production runtime and one rollback set. Candidate images/containers are disposable build outputs, not acceptance evidence; remove them once terminal and unreferenced, preserve receipt-referenced acceptance artifacts, and use `docs/development/cargo-target-policy.md` for candidate build targets and `docs/deployment/runtime-drift-policy.md` for standing image retention.
