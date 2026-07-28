import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import {
  buildListPagesMatrix,
  writeListPagesMatrix,
} from "../src/listpages-campaign-matrix.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(
  __dirname,
  "../scripts/build-listpages-campaign-matrix.mjs",
);

function jsonLine(row) {
  return `${JSON.stringify(row)}\n`;
}

async function writeFixtureInventory(root) {
  await fs.mkdir(root, { recursive: true });
  await fs.writeFile(
    path.join(root, "documentation-claims.jsonl"),
    [
      {
        id: "doc-modules:listpages-module:L107",
        claim: "++ Pagination",
      },
      {
        id: "doc-modules:listpages-module:L322",
        claim: 'argument="@URL|default-value" appends /name/value to the URL.',
      },
      {
        id: "doc-include:page-selection:L101",
        claim: 'Range selector: "others" means pages except current page',
      },
      {
        id: "doc-modules:listpages-module:L281",
        claim: "RSS feeds",
      },
      {
        id: "doc-wiki-syntax:links:L87",
        claim: "|| #_wantedpages || lists Wanted Pages ||",
        source: { page_fullname: "doc-wiki-syntax:links" },
      },
      {
        id: "doc-wiki-syntax:links:L90",
        claim: "|| #_editpage || opens Editor ||",
        source: { page_fullname: "doc-wiki-syntax:links" },
      },
      {
        id: "doc-wiki-syntax:links:L92",
        claim: "|| #_history || displays History ||",
        source: { page_fullname: "doc-wiki-syntax:links" },
      },
    ]
      .map(jsonLine)
      .join(""),
  );
  await fs.writeFile(
    path.join(root, "corpus-listpages-invocations.jsonl"),
    [
      {
        id: "en:alpha:L1:B0",
        branch: "en",
        page_fullname: "alpha",
        source_path: "/corpus/en/pages/alpha/source.wikidot.txt",
        line_start: 1,
        line_end: 3,
        balanced: true,
        malformed_reason: null,
        head: '[[module ListPages tags="+scp" limit="2"]]',
        body: "%%title%%",
        attributes: [
          { name: "tags", value: "+scp" },
          { name: "limit", value: "2" },
        ],
        duplicate_attributes: [],
        url_driven_attributes: [],
        template_variables: ["title"],
        body_sections: [],
        source_sha256: "a".repeat(64),
        semantic_cluster_key: "cluster-a",
      },
      {
        id: "en:beta:L5:B10",
        branch: "en",
        page_fullname: "beta",
        source_path: "/corpus/en/pages/beta/source.wikidot.txt",
        line_start: 5,
        line_end: 5,
        balanced: false,
        malformed_reason: "missing-module-close",
        head: '[[module ListPages category="fragment"]]',
        body: "",
        attributes: [{ name: "category", value: "fragment" }],
        duplicate_attributes: [],
        url_driven_attributes: [],
        template_variables: [],
        body_sections: [],
        source_sha256: "b".repeat(64),
        semantic_cluster_key: "cluster-b",
      },
    ]
      .map(jsonLine)
      .join(""),
  );
  await fs.writeFile(
    path.join(root, "corpus-listpages-clusters.json"),
    `${JSON.stringify({
      clusters: [
        {
          semantic_cluster_key: "cluster-a",
          count: 2,
          argument_signature: ["limit=2", "tags=+scp"],
          template_variables: ["title"],
          body_sections: [],
          first_provenance: {
            branch: "en",
            page_fullname: "alpha",
            source_path: "/corpus/en/pages/alpha/source.wikidot.txt",
            line_start: 1,
          },
        },
        {
          semantic_cluster_key: "cluster-b",
          count: 1,
          argument_signature: ["category=fragment"],
          template_variables: [],
          body_sections: [],
          first_provenance: {
            branch: "en",
            page_fullname: "beta",
            source_path: "/corpus/en/pages/beta/source.wikidot.txt",
            line_start: 5,
          },
        },
      ],
    })}\n`,
  );
}

test("matrix builder emits corpus, generated, navigation, and hash-magic lanes", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-matrix-"));
  const inventoryDir = path.join(root, "inventory");
  await writeFixtureInventory(inventoryDir);

  const matrix = await buildListPagesMatrix({ inventoryDir });
  assert.equal(matrix.summary.corpus_cluster_case_count, 2);
  assert.equal(matrix.summary.corpus_invocation_case_count, 2);
  assert.ok(matrix.summary.generated_case_count > 80);
  assert.ok(matrix.summary.navigation_case_count > 10);
  assert.equal(matrix.summary.hash_magic_case_count, 13);
  assert.ok(
    matrix.hash_magic_cases.some(
      (row) =>
        row.hash === "#_wantedpages" &&
        row.documented &&
        row.documentation_claim_ids.includes("doc-wiki-syntax:links:L87"),
    ),
  );
  assert.ok(
    matrix.hash_magic_cases.some(
      (row) =>
        row.hash === "#_tags" &&
        !row.documented &&
        row.documentation_claim_ids.length === 0,
    ),
  );
  assert.match(matrix.corpus_cluster_cases[0].source, /\[\[module ListPages/);
  assert.ok(
    matrix.generated_cases.some((row) =>
      row.documentation_claim_ids.includes("doc-include:page-selection:L101"),
    ),
  );
});

test("matrix writer and CLI create split case files", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-matrix-cli-"),
  );
  const inventoryDir = path.join(root, "inventory");
  const outputDir = path.join(root, "out");
  await writeFixtureInventory(inventoryDir);

  const matrix = await buildListPagesMatrix({ inventoryDir });
  await writeListPagesMatrix(matrix, outputDir);
  assert.ok(
    (await fs.stat(path.join(outputDir, "matrix-summary.json"))).isFile(),
  );
  assert.ok(
    (
      await fs.stat(path.join(outputDir, "generated-listpages-cases.jsonl"))
    ).isFile(),
  );

  const cliOutputDir = path.join(root, "cli-out");
  const { stdout } = await execFileAsync(process.execPath, [
    scriptPath,
    "--inventory-dir",
    inventoryDir,
    "--output-dir",
    cliOutputDir,
  ]);
  const summary = JSON.parse(stdout);
  assert.equal(summary.summary.corpus_invocation_case_count, 2);
  assert.ok(
    (await fs.stat(path.join(cliOutputDir, "navigation-cases.jsonl"))).isFile(),
  );
});
