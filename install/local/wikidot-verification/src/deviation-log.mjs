// Deviation-log and merge-readiness reporting for the local rendering lab.
//
// The iteration policy requires a deviation log during implementation: every
// intentional divergence from spec/plan is recorded with a reason and review
// state. A branch is merge-ready only when its validators pass and no
// deviation is unreviewed.

export const DEVIATION_LOG_SCHEMA = 'wikijump_local_lab.deviation_log.v1';
export const MERGE_READINESS_SCHEMA = 'wikijump_local_lab.merge_readiness.v1';

const REVIEW_STATES = new Set(['pending', 'approved', 'rejected', 'withdrawn']);

export function parseDeviationLog(text) {
  const entries = [];
  const errors = [];
  const lines = text.split('\n').filter((line) => line.trim() !== '');
  lines.forEach((line, index) => {
    let entry;
    try {
      entry = JSON.parse(line);
    } catch (error) {
      errors.push({ line: index + 1, error: `invalid JSON: ${error.message}` });
      return;
    }
    for (const field of ['id', 'date', 'summary', 'reason', 'review_state']) {
      if (!entry[field]) errors.push({ line: index + 1, error: `missing field: ${field}` });
    }
    if (entry.review_state && !REVIEW_STATES.has(entry.review_state)) {
      errors.push({ line: index + 1, error: `unknown review_state: ${entry.review_state}` });
    }
    entries.push(entry);
  });
  return { entries, errors };
}

/**
 * Compute merge readiness from validator verdict files and the deviation log.
 *
 * @param {object} input
 * @param {object[]} input.validators  [{name, exitCode, summary}] from prior runs
 * @param {object[]} input.deviations  parsed deviation entries
 * @param {object[]} [input.logErrors] deviation-log parse errors
 */
export function buildMergeReadiness({ runId, branch, validators, deviations, logErrors = [] }) {
  const blockers = [];
  for (const validator of validators) {
    if (validator.exitCode !== 0) {
      blockers.push({
        kind: 'validator-failing',
        name: validator.name,
        detail: `exit code ${validator.exitCode}`,
      });
    }
  }
  for (const error of logErrors) {
    blockers.push({ kind: 'deviation-log-invalid', detail: `line ${error.line}: ${error.error}` });
  }
  for (const deviation of deviations) {
    if (deviation.review_state === 'pending') {
      blockers.push({ kind: 'deviation-unreviewed', id: deviation.id, detail: deviation.summary });
    }
    if (deviation.review_state === 'rejected') {
      blockers.push({ kind: 'deviation-rejected', id: deviation.id, detail: deviation.summary });
    }
  }
  return {
    schema: MERGE_READINESS_SCHEMA,
    run_id: runId,
    branch,
    merge_ready: blockers.length === 0,
    blockers,
    validators: validators.map(({ name, exitCode }) => ({ name, exit_code: exitCode })),
    deviations: deviations.map(({ id, review_state }) => ({ id, review_state })),
  };
}
