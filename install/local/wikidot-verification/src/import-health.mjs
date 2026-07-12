// V1 import-health reporting for the local Wikidot rendering lab.
//
// Consumes apply-corpus-import-manifest.mjs JSONL logs (one action line per
// manifest row plus a trailing summary object) and produces a verdict:
// every row must reach an imported/done state or carry a classified failure
// code. Unclassified failures block (exit 2), mirroring the render-health
// taxonomy discipline.

export const IMPORT_HEALTH_SCHEMA = 'wikijump_local_lab.import_health.v1';

// Ordered: first match wins.
const FAILURE_CLASSIFIERS = [
  { code: 'auth-context-missing', pattern: /requires an authenticated request context|code":3106/ },
  { code: 'include-depth-exceeded', pattern: /include expansion exceeded maximum depth/ },
  { code: 'render-timeout', pattern: /timed out after \d+ms/ },
  { code: 'page-missing-after-create', pattern: /created page not found after page_create/ },
  { code: 'render-failed-on-create', pattern: /failed to run parse and render|failed to rerender/ },
  { code: 'rpc-error', pattern: /page_create failed/ },
];

const DONE_ACTIONS = new Set([
  'created',
  'created_db_snapshot_ready',
  'adopted',
  'created_snapshot_ready',
  'adopted_snapshot_ready',
  'skipped_existing_done',
]);

// Collision policy: an existing identical page is done; a snapshot mismatch is
// a classified failure requiring an explicit --replace-existing decision.
const COLLISION_ACTIONS = {
  collision_existing_page: { done: true, code: 'collision-existing-page' },
  collision_existing_snapshot_mismatch: { done: false, code: 'collision-snapshot-mismatch' },
};

export function classifyFailure(error) {
  const text = String(error ?? '');
  for (const { code, pattern } of FAILURE_CLASSIFIERS) {
    if (pattern.test(text)) return code;
  }
  return null;
}

function unknownActionDetail(action) {
  if (typeof action === 'string') return action;
  try {
    return JSON.stringify(action) ?? String(action);
  } catch {
    return Object.prototype.toString.call(action);
  }
}

export function parseImportLog(logText) {
  const rows = [];
  let summary = null;
  let buffer = '';
  for (const line of logText.split('\n')) {
    buffer += line;
    let parsed;
    try {
      parsed = JSON.parse(buffer);
    } catch {
      buffer += '\n';
      continue;
    }
    buffer = '';
    const isRecord = parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed);
    if (isRecord && Object.hasOwn(parsed, 'summary')) summary = parsed.summary;
    else if (isRecord && (Object.hasOwn(parsed, 'slug') || Object.hasOwn(parsed, 'action'))) {
      rows.push(parsed);
    }
  }
  return { rows, summary };
}

export function buildImportHealthVerdict({ runId, family, rows, summary = null }) {
  const failures = [];
  let done = 0;
  let unclassified = 0;
  const failureCounts = {};
  for (const row of rows) {
    if (typeof row?.slug !== 'string' || typeof row?.action !== 'string') {
      unclassified += 1;
      failures.push({
        slug: typeof row?.slug === 'string' ? row.slug : null,
        code: 'unclassified',
        detail: `invalid import row action ${unknownActionDetail(row?.action)}`,
      });
      failureCounts.unclassified = (failureCounts.unclassified ?? 0) + 1;
      continue;
    }
    if (DONE_ACTIONS.has(row.action)) {
      done += 1;
      continue;
    }
    if (Object.hasOwn(COLLISION_ACTIONS, row.action)) {
      const collision = COLLISION_ACTIONS[row.action];
      if (collision.done) done += 1;
      else {
        failures.push({ slug: row.slug, code: collision.code, detail: row.action });
        failureCounts[collision.code] = (failureCounts[collision.code] ?? 0) + 1;
      }
      continue;
    }
    if (row.action === 'failed') {
      const code = classifyFailure(row.error);
      if (code === null) {
        unclassified += 1;
        failures.push({ slug: row.slug, code: 'unclassified', detail: String(row.error).slice(0, 300) });
        failureCounts.unclassified = (failureCounts.unclassified ?? 0) + 1;
      } else {
        failures.push({ slug: row.slug, code, detail: String(row.error).slice(0, 300) });
        failureCounts[code] = (failureCounts[code] ?? 0) + 1;
      }
      continue;
    }
    unclassified += 1;
    failures.push({
      slug: row.slug,
      code: 'unclassified',
      detail: `unknown action ${unknownActionDetail(row.action)}`,
    });
    failureCounts.unclassified = (failureCounts.unclassified ?? 0) + 1;
  }

  const total = rows.length;
  const importRate = total === 0 ? 0 : done / total;
  return {
    verdict: {
      schema: IMPORT_HEALTH_SCHEMA,
      run_id: runId,
      family,
      aggregate: {
        family,
        rows_total: total,
        rows_done: done,
        import_rate: Number(importRate.toFixed(4)),
        unclassified,
        failure_counts: failureCounts,
      },
      failures,
      import_summary: summary,
    },
    exitCode: unclassified > 0 ? 2 : 0,
  };
}

export function applyThreshold(verdict, threshold) {
  if (threshold === null || threshold === undefined) return 0;
  return verdict.aggregate.import_rate >= threshold ? 0 : 1;
}
