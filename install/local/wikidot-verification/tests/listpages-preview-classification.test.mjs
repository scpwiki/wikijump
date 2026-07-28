import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  classifyListPagesPreviewDifferential,
} from "../src/listpages-preview-classification.mjs";
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

test("preview classifier separates oracle defects from fixture-state mismatches", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-classify-"));
  const referencesPath = path.join(root, "references.jsonl");
  const verdictPath = path.join(root, "verdict.json");
  const references = [
    reference(
      "invalid-range",
      '[[module ListPages range="bogus"]]\n%%title%%\n[[/module]]',
      '<div class="error-block">Invalid range argument.</div>',
    ),
    reference(
      "data",
      "[[module ListPages]]\n%%title%%\n[[/module]]",
      '<div class="list-pages-box"><div class="list-pages-item">live</div></div>',
    ),
  ];
  await fs.writeFile(
    referencesPath,
    references.map((row) => `${JSON.stringify(row)}\n`).join(""),
  );
  await fs.writeFile(verdictPath, JSON.stringify({
    cases: [
      {
        case_id: "invalid-range",
        status: "mismatch",
        live: { visible_text: "Invalid range argument." },
        local: { visible_text: "", html_sha256: "a".repeat(64) },
        comparison: {
          checks: {
            dom_tree: { status: "mismatch", local: [] },
          },
        },
      },
      {
        case_id: "data",
        status: "mismatch",
        live: { visible_text: "live" },
        local: { visible_text: "local", html_sha256: "b".repeat(64) },
        comparison: {
          checks: {
            dom_tree: {
              status: "mismatch",
              local: [{
                attrs: [{ name: "class", value: "list-pages-box" }],
                children: [],
              }],
            },
          },
        },
      },
    ],
  }));

  const result = await classifyListPagesPreviewDifferential({
    verdictPath,
    referencesPath,
  });
  assert.equal(result.summary.classifications["invalid-range-error"], 1);
  assert.equal(result.summary.classifications["inconclusive-fixture-data-state"], 1);
});

test("preview classifier recognizes executed wrapper-free modules", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-listpages-classify-"));
  const referencesPath = path.join(root, "references.jsonl");
  const verdictPath = path.join(root, "verdict.json");
  const source = [
    '[[module ListPages separate="no" wrapper="no"]]',
    "%%index%%. %%title%%",
    "[[/module]]",
  ].join("\n");
  const liveHtml = "<p>1. live one<br>2. live two</p><div class=\"pager\">pages</div>";
  await fs.writeFile(
    referencesPath,
    `${JSON.stringify(reference("wrapper-free", source, liveHtml))}\n`,
  );
  await fs.writeFile(verdictPath, JSON.stringify({
    cases: [{
      case_id: "wrapper-free",
      status: "mismatch",
      live: { visible_text: "1. live one\n2. live two" },
      local: { visible_text: "1. local", html_sha256: "c".repeat(64) },
      comparison: {
        checks: {
          dom_tree: {
            status: "mismatch",
            local: [{
              attrs: [],
              children: [{ type: "text", value: "1. local" }],
            }],
          },
        },
      },
    }],
  }));

  const result = await classifyListPagesPreviewDifferential({
    verdictPath,
    referencesPath,
  });
  assert.equal(
    result.cases[0].classification,
    "inconclusive-fixture-data-state",
  );
});
