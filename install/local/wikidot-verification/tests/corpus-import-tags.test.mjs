import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  batchShellCreatePageValues,
  upsertSnapshotSql,
} from "../scripts/apply-corpus-import-manifest.mjs";
import { buildCorpusImportManifest } from "../src/corpus-import-manifest.mjs";
import { writePage } from "./support/corpus-import-manifest-fixture.mjs";

test("corpus imports preserve the complete Wikidot tag set in serial and batch revisions", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "corpus-import-tags-"));
  const sourceTags = [
    "_cc",
    "_cc4",
    "_licensebox",
    "airborne",
    "dystopian",
    "esoteric-class",
    "gaseous",
    "horror",
    "illustrated",
    "scp",
    "transfiguration",
    "uncontained",
  ];
  writePage(root, "en", "scp-9506", {
    entityId: "95069506-9506-4506-8506-950695069506",
    meta: { tags: sourceTags },
    source: "SCP-9506 import fixture",
  });

  const [row] = buildCorpusImportManifest({
    corpusRoot: root,
    branch: "en",
    sourceSite: "scp-wiki",
    sourceBranch: "en",
  });
  const expectedTags = [...sourceTags].sort();
  assert.deepEqual(row.tags, expectedTags);

  const expectedSqlArray = `ARRAY[${expectedTags.map((tag) => `'${tag}'`).join(",")}]::text[]`;
  const serialSql = upsertSnapshotSql({}, row, 101, 201, 301);
  assert.ok(serialSql.includes(`tags = ${expectedSqlArray}`));

  const batchValues = batchShellCreatePageValues({}, [row], {
    categoryIds: new Map([["_default", 701]]),
    sourceTextEntityIds: new Set([row.source_entity_id]),
    textHash: () => "11".repeat(16),
    shellHash: () => "22".repeat(16),
  });
  assert.ok(batchValues.includes(expectedSqlArray));
});
