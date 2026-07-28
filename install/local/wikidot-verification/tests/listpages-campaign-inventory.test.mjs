import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import {
  buildCorpusListPagesInventory,
  buildDocumentationInventory,
  extractListPagesInvocationsFromSource,
} from "../src/listpages-campaign-inventory.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(
  __dirname,
  "../scripts/build-listpages-campaign-inventory.mjs",
);

async function writePage(root, fullname, source, meta = {}) {
  const pageDir = path.join(root, fullname);
  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), source);
  await fs.writeFile(
    path.join(pageDir, "meta.json"),
    `${JSON.stringify({ fullname, title: fullname, ...meta })}\n`,
  );
}

async function writeBranchPage(corpusRoot, branch, fullname, source) {
  const pageDir = path.join(corpusRoot, branch, "pages", fullname);
  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), source);
}

test("documentation inventory follows module docs, includes, and records missing references", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-docs-"));
  const docsRoot = path.join(root, "pages");
  await fs.mkdir(docsRoot, { recursive: true });

  await writePage(
    docsRoot,
    "doc-modules:start",
    "* [[[doc-modules:listpages-module | ListPages]]]\n* [[[doc-modules:pages-module | Pages]]]\n",
  );
  await writePage(
    docsRoot,
    "doc-modules:listpages-module",
    [
      "[[include doc-include:page-selection]]",
      "++ Pagination",
      '||~ Argument ||~ Meaning ||',
      '|| perPage || default is 20, maximum is 250. ||',
      '[[module ListPages category="@URL|design" separate="no"]]',
      "[[/module]]",
      "[[include doc-include:absent]]",
    ].join("\n"),
  );
  await writePage(
    docsRoot,
    "doc-include:page-selection",
    [
      "||~ Argument ||~ Meaning ||",
      '|| range || "others" means pages except current page ||',
    ].join("\n"),
  );
  await writePage(docsRoot, "doc-modules:pages-module", "Deprecated Pages module.\n");

  const inventory = await buildDocumentationInventory({ docsRoot });
  assert.equal(inventory.summary.inspected_document_count, 4);
  assert.ok(
    inventory.claims.some(
      (claim) =>
        claim.source.page_fullname === "doc-include:page-selection" &&
        claim.claim.includes('"others" means pages except current page'),
    ),
  );
  assert.ok(
    inventory.missing_references.some(
      (reference) => reference.target_fullname === "doc-include:absent",
    ),
  );
});

test("ListPages extraction preserves duplicates, URL attributes, body sections, and malformed modules", () => {
  const source = [
    '[[module ListPages tags="@URL|scp" tags="+featured" perPage="2"]]',
    "[[head]]H[[/head]]",
    "[[body]]%%title_linked%%[[/body]]",
    "[[/module]]",
    '[[module ListPages name="unterminated"',
  ].join("\n");

  const invocations = extractListPagesInvocationsFromSource({
    corpusRoot: "/corpus/en",
    branch: "en",
    pageFullname: "example",
    sourcePath: "/corpus/en/pages/example/source.wikidot.txt",
    source,
  });

  assert.equal(invocations.length, 2);
  assert.equal(invocations[0].balanced, true);
  assert.deepEqual(invocations[0].duplicate_attributes, ["tags"]);
  assert.deepEqual(invocations[0].url_driven_attributes, ["tags"]);
  assert.deepEqual(invocations[0].body_sections.sort(), ["body", "head"]);
  assert.deepEqual(invocations[0].template_variables, ["title_linked"]);
  assert.equal(invocations[1].balanced, false);
  assert.equal(invocations[1].malformed_reason, "unclosed-module-head");
});

test("corpus inventory scans selected branches and clusters semantic usages", async () => {
  const corpusRoot = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-corpus-"));
  await writeBranchPage(
    corpusRoot,
    "en",
    "alpha",
    '[[module ListPages tags="+scp" separate="no"]]%%title%%[[/module]]\n',
  );
  await writeBranchPage(
    corpusRoot,
    "en",
    "beta",
    '[[module ListPages separate="no" tags="+scp"]]%%title%%[[/module]]\n',
  );
  await writeBranchPage(corpusRoot, "jp", "gamma", "No module here.\n");

  const inventory = await buildCorpusListPagesInventory({
    corpusRoot,
    branches: ["en", "jp"],
  });

  assert.equal(inventory.summary.branch_count, 2);
  assert.equal(inventory.summary.invocation_count, 2);
  assert.equal(inventory.summary.cluster_count, 1);
  assert.equal(inventory.branches.find((branch) => branch.branch === "jp").invocation_count, 0);
});

test("campaign inventory CLI writes split machine-readable artifacts", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-cli-"));
  const docsRoot = path.join(root, "www", "pages");
  const corpusRoot = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  await fs.mkdir(docsRoot, { recursive: true });

  await writePage(docsRoot, "doc-modules:start", "[[[doc-modules:listpages-module]]]\n");
  await writePage(docsRoot, "doc-modules:listpages-module", "[[module ListPages]]\n");
  await writeBranchPage(
    corpusRoot,
    "en",
    "example",
    '[[module ListPages category="."]]%%fullname%%[[/module]]\n',
  );

  const { stdout } = await execFileAsync(process.execPath, [
    scriptPath,
    "--docs-root",
    docsRoot,
    "--corpus-root",
    corpusRoot,
    "--branch",
    "en",
    "--output-dir",
    outputDir,
  ]);

  const summary = JSON.parse(stdout);
  assert.equal(summary.corpus.invocation_count, 1);
  assert.ok((await fs.stat(path.join(outputDir, "documentation-claims.jsonl"))).isFile());
  assert.ok((await fs.stat(path.join(outputDir, "corpus-listpages-invocations.jsonl"))).isFile());
});
