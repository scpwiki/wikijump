import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  runListPagesPreviewDifferential,
  writePreviewDifferential,
} from "../src/listpages-preview-differential.mjs";
import { sha256 } from "../src/syntax-differential.mjs";

function reference(caseId, source, rawHtml) {
  return {
    schema: "wikijump_syntax_differential.wikidot_reference.v1",
    syntax_case: {
      schema: "wikijump_syntax_differential.syntax_case.v1",
      case_id: caseId,
      source,
      title: caseId,
      wikidot_observation_tier: "page-preview",
      local_execution_tier: "wikijump-runtime",
    },
    source_sha256: sha256(source),
    captured_at: "2026-07-27T00:00:00+00:00",
    provenance: {
      site: "sandbox-for-codex",
      site_domain: "sandbox-for-codex.wikidot.com",
      module: "edit/PagePreviewModule",
      wikidot_py_version: "4.4.1",
      wikidot_py_commit: "4af7c8eaec00a3e7a29fe502234e0aeeef968233",
      requirements_sha256: "c".repeat(64),
      authenticated: false,
      mutated: false,
    },
    raw_html: rawHtml,
    raw_html_sha256: sha256(rawHtml),
  };
}

async function writeReferences(filePath, rows) {
  await fs.writeFile(filePath, rows.map((row) => `${JSON.stringify(row)}\n`).join(""));
}

class FakeRpc {
  constructor(previews) {
    this.previews = previews;
  }

  async call(method, params) {
    if (method === "site_get") return { site_id: 7, slug: params.site };
    if (method === "wikidot_page_preview") {
      const value = this.previews.get(params.wikitext);
      if (value instanceof Error) throw value;
      return { body: value, styles: [] };
    }
    throw new Error(`unexpected method ${method}`);
  }
}

test("preview differential records matches and mismatches", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-preview-diff-"));
  const referencesPath = path.join(root, "references.jsonl");
  await writeReferences(referencesPath, [
    reference("match", "**x**", "<p>x</p>"),
    reference("mismatch", "**y**", "<p>live</p>"),
  ]);
  const verdict = await runListPagesPreviewDifferential({
    referencesPath,
    rpcUrl: "http://127.0.0.1:1/jsonrpc",
    siteSlug: "sandbox-for-codex",
    rpcClient: new FakeRpc(new Map([
      ["**x**", "<p>x</p>"],
      ["**y**", "<p>local</p>"],
    ])),
  });

  assert.equal(verdict.summary.counts.match, 1);
  assert.equal(verdict.summary.counts.mismatch, 1);
  assert.equal(verdict.summary.exit_code, 1);
  const mismatch = verdict.cases.find((row) => row.case_id === "mismatch");
  assert.equal(mismatch.comparison.checks.visible_text.live, "live");
  assert.equal(mismatch.comparison.checks.visible_text.local, "local");
});

test("preview differential records local errors and writes a verdict", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-preview-diff-error-"));
  const referencesPath = path.join(root, "references.jsonl");
  const output = path.join(root, "verdict.json");
  await writeReferences(referencesPath, [reference("boom", "source", "<p>live</p>")]);
  const verdict = await runListPagesPreviewDifferential({
    referencesPath,
    rpcUrl: "http://127.0.0.1:1/jsonrpc",
    siteSlug: "sandbox-for-codex",
    rpcClient: new FakeRpc(new Map([["source", new Error("boom")]])),
  });
  assert.equal(verdict.summary.counts["local-error"], 1);
  await writePreviewDifferential(verdict, output);
  assert.equal(JSON.parse(await fs.readFile(output, "utf8")).summary.counts["local-error"], 1);
});
