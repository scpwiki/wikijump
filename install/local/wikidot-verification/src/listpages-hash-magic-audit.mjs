import fs from "node:fs/promises";
import path from "node:path";

import { sha256 } from "./syntax-differential.mjs";

export const LISTPAGES_HASH_MAGIC_RECONCILIATION_SCHEMA =
  "wikijump_listpages_compat.hash_magic_reconciliation.v1";

const EXPECTED_OBSERVED_BEHAVIOR = new Map([
  ["#_wantedpages", "wanted-pages-panel-opened"],
  ["#_orphanedpages", "orphaned-pages-panel-opened"],
  ["#_draftpages", "draft-pages-panel-opened"],
  ["#_editpage", "permission-error-dialog-opened"],
  ["#_edittags", "permission-error-dialog-opened"],
  ["#_history", "history-panel-opened"],
  ["#_files", "files-panel-opened"],
  ["#_sitetools", "site-tools-panel-opened"],
  ["#_tags", "no-observed-effect"],
  ["#_discuss", "no-observed-effect"],
  ["#_edit", "no-observed-effect"],
  ["#_page-options", "no-observed-effect"],
  ["#_history/p/2", "history-panel-opened"],
]);

const LISTPAGES_SCOPE = new Map([
  ["#_history/p/2", "hash-magic-composition-not-listpages-pagination"],
]);

async function readJsonl(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  if (!text.trim()) return [];
  return text
    .trimEnd()
    .split(/\r?\n/u)
    .map((line) => JSON.parse(line));
}

function scopeFor(hash) {
  return LISTPAGES_SCOPE.get(hash) ?? "hash-magic-global-not-listpages";
}

function dispositionFor(row) {
  if (row.missing) return "capture-required";
  if (row.observed_behavior_classification === "capture-error") {
    return "recapture-required";
  }
  if (
    row.observed_behavior_classification ===
    "dom-or-network-changed-unclassified"
  ) {
    return "classify-required";
  }
  if (row.observed_behavior_classification !== row.expected_observed_behavior) {
    return "investigate-hash-magic-drift";
  }
  return "recorded-out-of-scope";
}

export async function reconcileListPagesHashMagicAudit({
  auditPath,
  matrixCasesPath,
}) {
  const auditText = await fs.readFile(auditPath, "utf8");
  const matrixText = await fs.readFile(matrixCasesPath, "utf8");
  const audit = JSON.parse(auditText);
  const matrixCases = await readJsonl(matrixCasesPath);
  const capturesById = new Map(
    audit.captures.map((capture) => [capture.case_id, capture]),
  );

  const duplicateCaptureIds = audit.captures
    .map((capture) => capture.case_id)
    .filter((id, index, ids) => ids.indexOf(id) !== index);

  const cases = matrixCases.map((matrixCase) => {
    const capture = capturesById.get(matrixCase.id);
    const expected = EXPECTED_OBSERVED_BEHAVIOR.get(matrixCase.hash) ?? null;
    const row = {
      case_id: matrixCase.id,
      hash: matrixCase.hash,
      documented: matrixCase.documented,
      documented_behavior: matrixCase.documented_behavior,
      documentation_claim_ids: matrixCase.documentation_claim_ids,
      listpages_scope: scopeFor(matrixCase.hash),
      expected_observed_behavior: expected,
      observed_behavior_classification:
        capture?.observed_behavior_classification ?? null,
      missing: !capture,
    };
    return {
      ...row,
      disposition: dispositionFor(row),
    };
  });

  const missing = cases.filter((row) => row.missing);
  const unresolved = cases.filter(
    (row) => row.disposition !== "recorded-out-of-scope",
  );

  const dispositions = {};
  const scopes = {};
  for (const row of cases) {
    dispositions[row.disposition] = (dispositions[row.disposition] ?? 0) + 1;
    scopes[row.listpages_scope] = (scopes[row.listpages_scope] ?? 0) + 1;
  }

  return {
    schema: LISTPAGES_HASH_MAGIC_RECONCILIATION_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      audit_path: auditPath,
      audit_sha256: sha256(auditText),
      matrix_cases_path: matrixCasesPath,
      matrix_cases_sha256: sha256(matrixText),
    },
    cases,
    duplicate_capture_ids: [...new Set(duplicateCaptureIds)].sort(),
    unresolved_case_ids: unresolved.map((row) => row.case_id),
    summary: {
      matrix_cases: matrixCases.length,
      captures: audit.captures.length,
      missing_captures: missing.length,
      duplicate_capture_ids: new Set(duplicateCaptureIds).size,
      unresolved: unresolved.length,
      listpages_implementation_required: 0,
      dispositions,
      scopes,
      exit_code:
        missing.length > 0 ||
        duplicateCaptureIds.length > 0 ||
        unresolved.length > 0
          ? 1
          : 0,
    },
  };
}

export async function writeListPagesHashMagicReconciliation(
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
