# Local authoring boundary

This note defines the local site boundary used by the Wikijump development stack for mirror QA and translation drafting.

## Canonical local sites

`scp-wiki` is the EN mirror site. It should reproduce real pages from `https://scp-wiki.wikidot.com/`. Local create/edit work must not target this site unless the task is explicitly mirror import or mirror repair.

`scp-jp` is the JP mirror or reserved JP site. It is not the default target for local draft authoring.

`scpaiueouiuiuiui` is the editable local corpus and manual-draft site. It is intentionally not a mirror of SCP-EN or SCP-JP. Local translation drafts, generated corpus pages, and manual experiments belong here unless a task explicitly says to import into a mirror.

`ai-translation` is a legacy name. It should not be used in canonical docs, scripts, tests, or generated local URLs.

## Authority model

The current local development model uses a local admin actor so the WSL owner can create and edit draft pages without depending on a finished Wikidot-like account flow.

That local authority must remain behind a replaceable seam. Page mutation code should continue to go through service/API boundaries and permission-aware paths. Do not add permanent deep bypasses inside page creation, page editing, revision persistence, or mirror import code.

Future work can replace the local admin actor with a Wikidot-like account/permission provider without changing which site owns editable drafts.

## Isolation rules

A slug created in `scpaiueouiuiuiui` must not appear in `scp-wiki` or `scp-jp` unless a separate explicit import step creates that page in a mirror site.

Mirror QA should treat a missing page in `scp-wiki` as an import/data coverage gap first. Visual parity bugs should be filed only after the mirror page exists and a browser comparison can inspect real rendered content.
