# Local Wikidot Compatibility Verifier

This directory contains a reusable local verification corpus for the Wikijump stack. It seeds Wikidot-shaped pages into the local `scp-wiki` site, then runs a browser proof matrix against the rendered Framerail pages.

## Prerequisites

Start the local stack from the repository root:

```bash
cd /home/roku/src/scpwiki/wikijump/install/local
docker compose up -d --build
```

The default verifier endpoints are:

```text
Deepwell JSON-RPC: http://127.0.0.1:2747/jsonrpc
Rendered site:      http://scp-wiki.wikijump.localhost:18443
```

## Seed Or Update The Corpus

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/seed-or-import.mjs \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v3-verifier-run
```

Useful environment variables:

```text
WIKIDOT_VERIFY_RPC_URL      JSON-RPC URL, default http://127.0.0.1:2747/jsonrpc
WIKIDOT_VERIFY_SITE_SLUG    site slug, default scp-wiki
WIKIDOT_VERIFY_ADMIN_EMAIL  seeded local admin email, default admin@wikijump
WIKIDOT_VERIFY_ADMIN_PASS   seeded local admin password, default wikijumpadmin1
```

## Run Browser Proof Matrix

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/browser-proof-matrix.mjs \
  --base-url http://scp-wiki.wikijump.localhost:18443 \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v3-browser-proof
```

The browser proof writes `browser-summary.json`, `fixture-results.tsv`, `screenshots/*.png`, and `network/*.json`. A non-zero exit means at least one required compatibility fixture failed.

## Discover A Real Corpus

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/corpus-discover.mjs \
  --corpus /home/roku/src/Rokurolize/scp-wiki-translation/corpus/en \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-plan-state \
  --canary-count 100
```

The discovery command writes `corpus-file-inventory.tsv`, `corpus-manifest.tsv`, `canary-pages.tsv`, and summary JSON/Markdown. `corpus-manifest.tsv` is the input for batch render and one-file preview commands.

## Batch Render Real Corpus Pages

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/corpus-render-batch.mjs \
  --manifest /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-plan-state/corpus-manifest.tsv \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-render-batch-0000-0024-deps \
  --offset 0 \
  --limit 25 \
  --batch-size 25 \
  --rpc-url http://127.0.0.1:12748/jsonrpc \
  --preload-dependencies \
  --max-dependencies 80
```

The batch command writes page-level diagnostics, rendered HTML, `compatibility-results.tsv`, `corpus-batch-ledger.tsv`, and `batch-summary.json`.

## Preview One Source File

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/preview-source.mjs \
  --source /home/roku/src/Rokurolize/scp-wiki-translation/corpus/en/pages/11-mr-feather/source.wikidot.txt \
  --manifest /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-plan-state/corpus-manifest.tsv \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-preview-smoke-11-mr-feather \
  --rpc-url http://127.0.0.1:12748/jsonrpc \
  --json
```

The preview command writes `preview-result.json` and rendered HTML. The JSON contract includes the input source hash, preview slug, HTML path, diagnostics, dependency list, asset list, Wikijump page reference, and timing.

## Preview A Canary Batch

```bash
cd /home/roku/src/scpwiki/wikijump
node install/local/wikidot-verification/scripts/preview-batch.mjs \
  --input /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-plan-state/canary-pages.tsv \
  --manifest /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-plan-state/corpus-manifest.tsv \
  --output-dir /home/roku/codex-thread-workspaces/019ebf4b-585e-7b93-bd6d-cdba089c8084/artifacts/wikijump/v5-preview-canary-100 \
  --offset 0 \
  --limit 100 \
  --rpc-url http://127.0.0.1:12748/jsonrpc \
  --slug-prefix preview-canary-
```

The preview batch command writes `preview-results.tsv`, `preview-summary.json`, and per-page `preview-result.json` and HTML artifacts under `pages/`. The summary includes severity counts and p50/p95 timing.

## Required Proof Pages

The manifest defines the minimum required compatibility surface:

- `scp-9506` regression page with twenty local image loads.
- `fixture-source-basic` for basic source parsing and formatting.
- `fixture-include-host` for include and component substitution.
- `fixture-listpages-index` for ListPages/module behavior.
- `fixture-theme-nav-css` for Wikidot shell, navigation, and internal CSS behavior.
- `fixture-assets-network` for page-file assets and network isolation.
- `fixture-metadata-tags-edit` for create/edit/save/tag and parent workflow proof.
