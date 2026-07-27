# Wikidot feature specifications

This directory is an exhaustive, documentation-derived implementation inventory for the frozen Wikidot corpus at `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages`.

- `catalog.json` is the authoritative machine-readable feature index.
- `CATALOG.md` is the human-readable index.
- `source-coverage.json` proves that all 1,806 corpus pages were enumerated and classified.
- `specifications/` contains exactly one English Markdown specification for every catalog item.
- `IMPLEMENTATION_PROMPT.md` instructs a coding agent to implement the complete catalog using vertical-slice TDD.

## Interpretation rules

1. A corpus page is not automatically a feature. The 1,560 `community-sites:*` pages, for example, are structured records created through one directory/data-form feature.
2. Redirects, indexes, navigation fragments, marketing repetitions, policies, and runtime composition pages are retained in `source-coverage.json`; relevant pages are attached to canonical feature specs as supporting evidence.
3. Every normative source extract retains its exact corpus page, original line numbers, and complete-file SHA-256.
4. Documentation status matters. `invocation-only`, `high-level-documentation`, and `partially-documented` specs identify real features but do not authorize invented behavior.
5. This snapshot is a specification-discovery input. When a reproducible live Wikidot observation conflicts with it, record both and implement live behavior.

## Regeneration

```bash
node scripts/generate-wikidot-specifications.mjs
node scripts/generate-wikidot-specifications.mjs --check
```

Set `WIKIDOT_DOCUMENTATION_CORPUS` only when regenerating from a different checkout of the same corpus layout.
