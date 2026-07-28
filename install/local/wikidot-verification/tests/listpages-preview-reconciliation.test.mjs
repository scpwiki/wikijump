import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  LISTPAGES_PREVIEW_RECONCILIATION_SCHEMA,
  reconcileListPagesPreviewClassification,
} from "../src/listpages-preview-reconciliation.mjs";
import { main as reconcileCli } from "../scripts/reconcile-listpages-preview-differential.mjs";

function classification(cases) {
  return {
    schema: "wikijump_listpages_compat.preview_classification.v1",
    inputs: {
      verdict_path: "/tmp/verdict.json",
      references_path: "/tmp/references.jsonl",
    },
    cases,
  };
}

async function writeClassification(root, cases) {
  const classificationPath = path.join(root, "classification.json");
  await fs.writeFile(
    classificationPath,
    `${JSON.stringify(classification(cases), null, 2)}\n`,
  );
  return classificationPath;
}

test("reconciles matched and fixture-data preview mismatches as non-actionable", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-preview-reconcile-"),
  );
  try {
    const classificationPath = await writeClassification(root, [
      {
        case_id: "match",
        differential_status: "match",
        classification: "matched",
        disposition: "none",
        rationale: "Canonical DOM and visible text match.",
      },
      {
        case_id: "fixture",
        differential_status: "mismatch",
        classification: "inconclusive-fixture-data-state",
        disposition: "replay-synchronized-fixture",
        rationale: "Both runtimes execute ListPages against different pages.",
      },
    ]);

    const reconciliation = await reconcileListPagesPreviewClassification({
      classificationPath,
    });

    assert.equal(
      reconciliation.schema,
      LISTPAGES_PREVIEW_RECONCILIATION_SCHEMA,
    );
    assert.equal(reconciliation.summary.exit_code, 0);
    assert.equal(reconciliation.summary.actionable, 0);
    assert.equal(reconciliation.summary.fixture_data_state, 1);
    assert.deepEqual(reconciliation.actionable_case_ids, []);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("keeps parser and renderer dispositions actionable", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-preview-reconcile-"),
  );
  try {
    const classificationPath = await writeClassification(root, [
      {
        case_id: "parser",
        differential_status: "mismatch",
        classification: "live-parser-accepts-local-preserves",
        disposition: "minimize-parser",
        rationale: "Live executes the module.",
      },
      {
        case_id: "render",
        differential_status: "mismatch",
        classification: "listpages-render-shape-divergence",
        disposition: "investigate-renderer",
        rationale: "Live emits a ListPages container.",
      },
    ]);

    const reconciliation = await reconcileListPagesPreviewClassification({
      classificationPath,
    });

    assert.equal(reconciliation.summary.exit_code, 1);
    assert.equal(reconciliation.summary.actionable, 2);
    assert.deepEqual(reconciliation.actionable_case_ids, ["parser", "render"]);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("writes reconciliation artifacts without replacement", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-preview-reconcile-"),
  );
  try {
    const classificationPath = await writeClassification(root, [
      {
        case_id: "fixture",
        differential_status: "mismatch",
        classification: "inconclusive-fixture-data-state",
        disposition: "replay-synchronized-fixture",
        rationale: "Both runtimes execute ListPages against different pages.",
      },
    ]);
    const output = path.join(root, "reconciliation.json");

    assert.equal(
      await reconcileCli([
        "node",
        "script",
        "--classification",
        classificationPath,
        "--output",
        output,
      ]),
      0,
    );
    const reconciliation = JSON.parse(await fs.readFile(output, "utf8"));
    assert.equal(
      reconciliation.schema,
      LISTPAGES_PREVIEW_RECONCILIATION_SCHEMA,
    );
    await assert.rejects(
      () =>
        reconcileCli([
          "node",
          "script",
          "--classification",
          classificationPath,
          "--output",
          output,
        ]),
      /EEXIST/u,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
