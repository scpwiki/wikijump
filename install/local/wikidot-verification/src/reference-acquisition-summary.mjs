import { stableStringify } from "./canonical-json.mjs";

function assertExactArray(actual, expected, label) {
  if (
    !Array.isArray(actual) ||
    stableStringify(actual) !== stableStringify(expected)
  ) {
    throw new Error(`${label} does not match the manifest`);
  }
}

export function validateAcquisitionSummary(summary, context) {
  const visibilityCounts = {};
  for (const row of context.inputRows) {
    if (row.source_browser_visibility !== undefined) {
      visibilityCounts[row.source_browser_visibility] =
        (visibilityCounts[row.source_browser_visibility] ?? 0) + 1;
    }
  }
  const expected = {
    attachment_count: context.attachmentCount,
    attachment_page_count: context.attachmentPageCount,
    first_fullname: context.rows[0]?.fullname ?? null,
    last_fullname: context.rows.at(-1)?.fullname ?? null,
    manifest_sha256: context.manifestSha256,
    parent_count: context.inputRows.filter(
      (row) => row.parent_fullname !== null,
    ).length,
    required_browser_count: context.inputRows.filter(
      (row) => row.required_browser === true,
    ).length,
    row_count: context.rows.length,
    source_required_actor_count: context.inputRows.filter(
      (row) =>
        row.source_required_actor !== undefined &&
        row.source_required_actor !== null,
    ).length,
  };
  for (const [field, value] of Object.entries(expected)) {
    if (summary[field] !== value) {
      throw new Error(
        `summary.${field} mismatch: expected ${value}, got ${summary[field]}`,
      );
    }
  }
  assertExactArray(
    summary.source_sites,
    [context.sourceSite],
    "summary.source_sites",
  );
  assertExactArray(
    summary.source_branches,
    [context.sourceBranch],
    "summary.source_branches",
  );
  if (
    stableStringify(summary.source_browser_visibility_counts) !==
    stableStringify(visibilityCounts)
  ) {
    throw new Error(
      "summary.source_browser_visibility_counts does not match the manifest",
    );
  }
}
