import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import {
  exportPreviewCases,
} from "../scripts/export-listpages-preview-cases.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(
  __dirname,
  "../scripts/export-listpages-preview-cases.mjs",
);

async function writeJsonl(filePath, rows) {
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  await fs.writeFile(filePath, rows.map((row) => `${JSON.stringify(row)}\n`).join(""));
}

test("exports generated matrix rows as syntax preview cases with campaign provenance", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-preview-export-"));
  const matrixDir = path.join(root, "matrix");
  const output = path.join(root, "cases.jsonl");
  await writeJsonl(path.join(matrixDir, "generated-listpages-cases.jsonl"), [
    {
      id: "lpgen-0001-category",
      origin: "generated",
      label: "category selector",
      source: '[[module ListPages category="."]]\n%%title%%\n[[/module]]',
      dimensions: ["selector", "category"],
      documentation_claim_ids: ["doc-include:page-selection:L34"],
    },
    {
      id: "lpgen-0002-tags",
      origin: "generated",
      label: "tags selector",
      source: '[[module ListPages tags="+scp"]]\n%%title%%\n[[/module]]',
      dimensions: ["selector", "tags"],
      documentation_claim_ids: [],
    },
  ]);

  const summary = await exportPreviewCases({
    matrixDir,
    lanes: ["generated"],
    limit: 1,
    output,
  });
  assert.equal(summary.case_count, 1);
  const record = JSON.parse((await fs.readFile(output, "utf8")).trim());
  assert.equal(record.schema, "wikijump_syntax_differential.syntax_case.v1");
  assert.equal(record.local_execution_tier, "wikijump-runtime");
  assert.deepEqual(record.campaign_matrix.documentation_claim_ids, [
    "doc-include:page-selection:L34",
  ]);
});

test("export CLI writes the requested lane", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-preview-export-cli-"));
  const matrixDir = path.join(root, "matrix");
  const output = path.join(root, "cases.jsonl");
  await writeJsonl(path.join(matrixDir, "corpus-cluster-cases.jsonl"), [
    {
      id: "lpcorpus-0001",
      origin: "corpus-cluster-representative",
      source: "[[module ListPages]]%%title%%[[/module]]",
      dimensions: ["corpus"],
      provenance: { branch: "en", page_fullname: "example" },
    },
  ]);

  const { stdout } = await execFileAsync(process.execPath, [
    scriptPath,
    "--matrix-dir",
    matrixDir,
    "--lane",
    "corpus-cluster",
    "--output",
    output,
  ]);
  const summary = JSON.parse(stdout);
  assert.equal(summary.case_count, 1);
  const record = JSON.parse((await fs.readFile(output, "utf8")).trim());
  assert.equal(record.case_id, "lpcorpus-0001");
  assert.equal(record.campaign_matrix.provenance.branch, "en");
});
