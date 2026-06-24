# rokurokubi reservation extraction

This directory contains the current reproducible extraction for reservation rows where `翻訳者名` is `rokurokubi`, case-insensitively.

Regenerate from a saved Google Sheet CSV export with:

```bash
node install/local/wikidot-verification/scripts/extract-rokurokubi-reservations.mjs \
  --source /path/to/source-sheet-gid1325361212.csv \
  --output install/local/wikidot-verification/artifacts/rokurokubi-reservations/rokurokubi-reservations-with-wikijump-mirror.csv \
  --manifest install/local/wikidot-verification/artifacts/rokurokubi-reservations/manifest.json \
  --source-label 'https://docs.google.com/spreadsheets/d/1_J_rte3pfZ8uEbq8r1xqJycU89XkO5Z6yhW-2SkeNak/edit?gid=1325361212#gid=1325361212'
```

The current snapshot maps SCP Wiki source URLs to `http://scp-wiki.wikijump.localhost:18443/<slug>` for browser QA input. It does not assert that the local mirror page exists or has parity; browser QA issues own that proof.
