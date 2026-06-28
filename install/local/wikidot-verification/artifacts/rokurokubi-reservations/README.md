# rokurokubi reservation extraction

This directory contains the reproducible target-inventory ledger for translation reservation rows where `翻訳者名` is `rokurokubi`, case-insensitively.

The current inventory is intentionally broader than the active reservation tab. It includes:

- active sheet: `Main`, gid `1325361212`
- expired/deleted sheet: `期限切れ削除済`, gid unknown from the available XLSX export

The current ledger has 27 `rokurokubi` rows after deduplication. Of these, 26 rows map to EN SCP Wiki source pages and belong to the local `scp-wiki` mirror-open gate. One CN row is preserved as source-sheet provenance but is not an EN mirror target.

Regenerate from one saved CSV export with:

```bash
node install/local/wikidot-verification/scripts/extract-rokurokubi-reservations.mjs \
  --source /path/to/source-sheet-gid1325361212.csv \
  --output install/local/wikidot-verification/artifacts/rokurokubi-reservations/rokurokubi-reservations-with-wikijump-mirror.csv \
  --manifest install/local/wikidot-verification/artifacts/rokurokubi-reservations/manifest.json \
  --source-role active \
  --source-name Main \
  --source-gid 1325361212 \
  --source-label 'https://docs.google.com/spreadsheets/d/1_J_rte3pfZ8uEbq8r1xqJycU89XkO5Z6yhW-2SkeNak/edit?gid=1325361212#gid=1325361212'
```

Regenerate from multiple saved tab CSV exports with a sheet manifest:

```bash
node install/local/wikidot-verification/scripts/extract-rokurokubi-reservations.mjs \
  --sheet-manifest /path/to/rokurokubi-source-sheets.json \
  --output install/local/wikidot-verification/artifacts/rokurokubi-reservations/rokurokubi-reservations-with-wikijump-mirror.csv \
  --manifest install/local/wikidot-verification/artifacts/rokurokubi-reservations/manifest.json
```

Example sheet manifest:

```json
{
  "sheets": [
    {
      "csv": "/path/to/main.csv",
      "role": "active",
      "name": "Main",
      "gid": "1325361212",
      "label": "https://docs.google.com/spreadsheets/d/1_J_rte3pfZ8uEbq8r1xqJycU89XkO5Z6yhW-2SkeNak/edit?gid=1325361212#gid=1325361212"
    },
    {
      "csv": "/path/to/expired.csv",
      "role": "expired",
      "name": "期限切れ削除済",
      "gid": "unknown",
      "label": "Google Sheets tab name discovered from the workbook export; gid was not exposed by the XLSX snapshot"
    }
  ]
}
```

The canonical local mirror URL in this ledger is `https://scp-wiki.wikijump.localhost/<slug>`. The old `http://scp-wiki.wikijump.localhost:18443/<slug>` URL is not used in this artifact because previous browser evidence showed the local Caddy route being exercised at the HTTPS origin.

This artifact records target inventory only. It does not prove that the mirror page exists or has rendering parity. A row is complete for the WBS only after browser QA shows that the corresponding `scp-wiki` mirror URL opens actual article content, not `Page not found`, `UNTRANSLATED:Svelte Error`, or a generic app shell.
