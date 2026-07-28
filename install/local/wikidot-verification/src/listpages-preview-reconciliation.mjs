import fs from "node:fs/promises";
import path from "node:path";

import { sha256 } from "./syntax-differential.mjs";

export const LISTPAGES_PREVIEW_RECONCILIATION_SCHEMA =
  "wikijump_listpages_compat.preview_reconciliation.v1";

const NON_ACTIONABLE_DISPOSITIONS = new Set([
  "none",
  "replay-synchronized-fixture",
]);

function caseStatus(disposition) {
  return NON_ACTIONABLE_DISPOSITIONS.has(disposition)
    ? "non-actionable"
    : "actionable";
}

export async function reconcileListPagesPreviewClassification({
  classificationPath,
}) {
  const classificationText = await fs.readFile(classificationPath, "utf8");
  const classification = JSON.parse(classificationText);
  const cases = classification.cases.map((row) => ({
    case_id: row.case_id,
    differential_status: row.differential_status,
    classification: row.classification,
    disposition: row.disposition,
    status: caseStatus(row.disposition),
    rationale: row.rationale,
  }));

  const actionableCases = cases.filter((row) => row.status === "actionable");
  const dispositionCounts = {};
  const statusCounts = {};
  const classificationCounts = {};
  for (const row of cases) {
    dispositionCounts[row.disposition] =
      (dispositionCounts[row.disposition] ?? 0) + 1;
    statusCounts[row.status] = (statusCounts[row.status] ?? 0) + 1;
    classificationCounts[row.classification] =
      (classificationCounts[row.classification] ?? 0) + 1;
  }

  return {
    schema: LISTPAGES_PREVIEW_RECONCILIATION_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      classification_path: classificationPath,
      classification_sha256: sha256(classificationText),
      differential_path: classification.inputs?.verdict_path ?? null,
      references_path: classification.inputs?.references_path ?? null,
    },
    cases,
    actionable_case_ids: actionableCases.map((row) => row.case_id),
    summary: {
      total: cases.length,
      actionable: actionableCases.length,
      non_actionable: cases.length - actionableCases.length,
      fixture_data_state:
        classificationCounts["inconclusive-fixture-data-state"] ?? 0,
      matched: classificationCounts.matched ?? 0,
      statuses: statusCounts,
      classifications: classificationCounts,
      dispositions: dispositionCounts,
      exit_code: actionableCases.length > 0 ? 1 : 0,
    },
  };
}

export async function writeListPagesPreviewReconciliation(
  reconciliation,
  outputPath,
) {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(
    outputPath,
    `${JSON.stringify(reconciliation, null, 2)}\n`,
    {
      encoding: "utf8",
      flag: "wx",
    },
  );
}
