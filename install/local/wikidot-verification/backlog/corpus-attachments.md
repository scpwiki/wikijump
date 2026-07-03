# Corpus Attachment Fixtures

Imported Wikidot page-local files are not durable Wikijump seed data. They belong in the `scp-wiki-translation` corpus beside the page source that references them.

Per page, captured attachment bytes use this layout:

```text
<corpus-root>/<branch>/pages/<slug>/source.wikidot.txt
<corpus-root>/<branch>/pages/<slug>/meta.json
<corpus-root>/<branch>/pages/<slug>/files.json
<corpus-root>/<branch>/pages/<slug>/files/<filename>
```

`files.json` is an array. Each entry records:

- `filename`: decoded Wikidot file name used for Deepwell `file_create`
- `original_url`: canonical Wikidot `http` or `https` URL
- `wikidot_path`: `/local--files/<page>/<filename>` path from that URL
- `path`: page-relative byte path below `files/`
- `sha256`: lowercase SHA-256 of the captured bytes
- `mime`: MIME guess used for operator inspection
- `size`: byte length

Use `scripts/capture-corpus-files.mjs --corpus-root <scp-wiki-translation/corpus> --branch en --slug <slug>` to populate the corpus attachment layout. Use `scripts/build-corpus-import-manifest.mjs` to carry those entries into an import manifest, then `scripts/apply-corpus-import-manifest.mjs --session-token <token>` to upload them into a local Deepwell runtime through `blob_upload` and `file_create`.
