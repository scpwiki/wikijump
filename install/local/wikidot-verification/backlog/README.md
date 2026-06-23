# Reserved Fixture Backlog

This artifact tracks a deterministic fixture backlog for Wikidot parity coverage that is reserved for Issue #9 planning.

Inputs are limited to local corpus evidence under `/home/roku/src/Rokurolize/scp-wiki-translation/corpus/en/pages`.

## Deterministic inputs

- Use exactly one path per row in `source_path`.
- Keep `priority` numeric and sorted ascending for reproducibility.
- Keep required fixture list stable and always include at least these slugs:
  - `scp-3352`
  - `scp-8980`
  - `scp-anthology-2024`
  - `scp-9506`

## Validation

Validation is implemented by `install/local/wikidot-verification/backlog/validate-reserved-fixture-backlog.sh` and checks:

- TSV header matches schema
- Unique `slug`
- Required fixture slugs are present
- `local_corpus_status == present` rows have existing `source_path`
- `recommended_issue` is within allowed set
- Schema and required columns are intact

## Artifact purpose

The backlog is planning infrastructure only. It does not change renderer behavior and is safe to iterate on independently from implementation PRs.
