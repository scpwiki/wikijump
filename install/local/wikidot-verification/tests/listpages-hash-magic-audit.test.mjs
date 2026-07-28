import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  LISTPAGES_HASH_MAGIC_RECONCILIATION_SCHEMA,
  reconcileListPagesHashMagicAudit,
} from "../src/listpages-hash-magic-audit.mjs";
import { main as reconcileCli } from "../scripts/reconcile-listpages-hash-magic-audit.mjs";

function matrixCase(id, hash, documented = false) {
  return {
    id,
    hash,
    documented,
    documented_behavior: documented ? "documented behavior" : null,
    documentation_claim_ids: documented ? ["doc:L1"] : [],
  };
}

async function writeFixture(root, { matrixCases, captures }) {
  const matrixCasesPath = path.join(root, "hash-magic.jsonl");
  const auditPath = path.join(root, "audit.json");
  await fs.writeFile(
    matrixCasesPath,
    matrixCases.map((row) => `${JSON.stringify(row)}\n`).join(""),
  );
  await fs.writeFile(
    auditPath,
    `${JSON.stringify({
      schema: "wikijump_listpages_compat.hash_magic_live_audit.v1",
      captures,
    })}\n`,
  );
  return { matrixCasesPath, auditPath };
}

test("reconciles captured Hash Magic behavior as recorded and out of ListPages implementation scope", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-hash-magic-"),
  );
  try {
    const { matrixCasesPath, auditPath } = await writeFixture(root, {
      matrixCases: [
        matrixCase("wanted", "#_wantedpages", true),
        matrixCase("history-pager", "#_history/p/2"),
      ],
      captures: [
        {
          case_id: "wanted",
          observed_behavior_classification: "wanted-pages-panel-opened",
        },
        {
          case_id: "history-pager",
          observed_behavior_classification: "history-panel-opened",
        },
      ],
    });

    const reconciliation = await reconcileListPagesHashMagicAudit({
      auditPath,
      matrixCasesPath,
    });

    assert.equal(
      reconciliation.schema,
      LISTPAGES_HASH_MAGIC_RECONCILIATION_SCHEMA,
    );
    assert.equal(reconciliation.summary.exit_code, 0);
    assert.equal(reconciliation.summary.listpages_implementation_required, 0);
    assert.equal(
      reconciliation.summary.scopes[
        "hash-magic-composition-not-listpages-pagination"
      ],
      1,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("flags missing, errored, unclassified, and drifted Hash Magic cases", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-hash-magic-"),
  );
  try {
    const { matrixCasesPath, auditPath } = await writeFixture(root, {
      matrixCases: [
        matrixCase("missing", "#_wantedpages", true),
        matrixCase("error", "#_files", true),
        matrixCase("unclassified", "#_history", true),
        matrixCase("drift", "#_draftpages", true),
      ],
      captures: [
        { case_id: "error", observed_behavior_classification: "capture-error" },
        {
          case_id: "unclassified",
          observed_behavior_classification:
            "dom-or-network-changed-unclassified",
        },
        {
          case_id: "drift",
          observed_behavior_classification: "no-observed-effect",
        },
      ],
    });

    const reconciliation = await reconcileListPagesHashMagicAudit({
      auditPath,
      matrixCasesPath,
    });

    assert.equal(reconciliation.summary.exit_code, 1);
    assert.deepEqual(reconciliation.unresolved_case_ids, [
      "missing",
      "error",
      "unclassified",
      "drift",
    ]);
    assert.equal(reconciliation.summary.dispositions["capture-required"], 1);
    assert.equal(reconciliation.summary.dispositions["recapture-required"], 1);
    assert.equal(reconciliation.summary.dispositions["classify-required"], 1);
    assert.equal(
      reconciliation.summary.dispositions["investigate-hash-magic-drift"],
      1,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("writes Hash Magic reconciliation artifacts without replacement", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-hash-magic-"),
  );
  try {
    const { matrixCasesPath, auditPath } = await writeFixture(root, {
      matrixCases: [matrixCase("files", "#_files", true)],
      captures: [
        {
          case_id: "files",
          observed_behavior_classification: "files-panel-opened",
        },
      ],
    });
    const output = path.join(root, "reconciliation.json");
    assert.equal(
      await reconcileCli([
        "node",
        "script",
        "--audit",
        auditPath,
        "--matrix-cases",
        matrixCasesPath,
        "--output",
        output,
      ]),
      0,
    );
    const reconciliation = JSON.parse(await fs.readFile(output, "utf8"));
    assert.equal(
      reconciliation.schema,
      LISTPAGES_HASH_MAGIC_RECONCILIATION_SCHEMA,
    );
    await assert.rejects(
      () =>
        reconcileCli([
          "node",
          "script",
          "--audit",
          auditPath,
          "--matrix-cases",
          matrixCasesPath,
          "--output",
          output,
        ]),
      /EEXIST/u,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
