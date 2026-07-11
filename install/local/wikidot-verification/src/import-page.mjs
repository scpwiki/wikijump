// Pure helpers for the single-page on-demand import runner
// (scripts/import-page.mjs). Kept side-effect free so the ordering,
// classification, and batching rules stay unit-testable without a runtime.

import fs from 'node:fs';
import path from 'node:path';

export const CORPUS_PAGE_REQUIRED_FILES = ['source.wikidot.txt', 'meta.json', 'entity_id.txt'];

// Corpus presence check for one page directory. Returns
// { status: 'ok' } or { status: 'missing' } (no directory) or
// { status: 'incomplete', missing_files: [...] } (directory without the
// files the manifest builder requires).
export function corpusPageStatus(corpusRoot, branch, fullname) {
  const pageDir = path.join(corpusRoot, branch, 'pages', fullname);
  if (!fs.existsSync(pageDir)) return { status: 'missing' };
  const missingFiles = CORPUS_PAGE_REQUIRED_FILES.filter(
    (name) => !fs.existsSync(path.join(pageDir, name)),
  );
  if (missingFiles.length > 0) return { status: 'incomplete', missing_files: missingFiles };
  return { status: 'ok' };
}

// Closure reports label dependencies as `${prefix}:${slug}` where the prefix
// is an inventory fixture family ("EN") or a site ("scp-wiki"); the slug
// itself may contain colons ("component:license-box").
export function slugFromDependencyLabel(label) {
  const separator = label.indexOf(':');
  return separator === -1 ? label : label.slice(separator + 1);
}

// Flatten dependency-closure reports into an ordered, deduplicated import
// plan: dependencies first (in each report's import_order), requested pages
// last. Out-of-bundle dependencies are surfaced, never imported.
export function planImportSet({ requestedSlugs, closureReports = [] }) {
  const requested = new Set(requestedSlugs);
  const dependencySlugs = [];
  const seen = new Set(requestedSlugs);
  const outOfBundle = [];
  const closureStatuses = {};
  for (const report of closureReports) {
    closureStatuses[slugFromDependencyLabel(report.fixture_id)] = report.status;
    for (const label of report.import_order ?? []) {
      const slug = slugFromDependencyLabel(label);
      if (seen.has(slug)) continue;
      seen.add(slug);
      dependencySlugs.push(slug);
    }
    for (const dep of report.dependencies?.out_of_bundle ?? []) {
      outOfBundle.push({ page: slugFromDependencyLabel(report.fixture_id), ...dep });
    }
  }
  return {
    dependencySlugs,
    importSlugs: [...dependencySlugs, ...requested],
    outOfBundle,
    closureStatuses,
  };
}

// Deepwell login tokens expire after roughly five minutes, so apply runs are
// batched with a fresh login per batch (same pattern as the P2 pilot).
export function batchSlugs(slugs, batchSize) {
  if (!Number.isInteger(batchSize) || batchSize < 1) {
    throw new Error('batch size must be a positive integer');
  }
  const batches = [];
  for (let start = 0; start < slugs.length; start += batchSize) {
    batches.push(slugs.slice(start, start + batchSize));
  }
  return batches;
}

export async function resolveSessionToken({ sessionToken, rpcUrl, login }) {
  if (sessionToken) return sessionToken;
  return await login(rpcUrl);
}

export function buildApplyInvocation({
  batchPath,
  rpcUrl,
  siteId,
  attachmentUserId,
  sessionToken,
  adoptExisting = false,
  dbContainer = null,
  dryRun = false,
}) {
  const scriptArgs = [
    '--manifest', batchPath,
    '--create-mode', 'rpc',
    '--api-url', rpcUrl,
    '--site-id', String(siteId),
    '--skip-existing-done',
    '--attachment-user-id', attachmentUserId,
    '--presign-host-alias', 'files=127.0.0.1',
  ];
  if (adoptExisting) scriptArgs.push('--adopt-existing');
  if (dbContainer) scriptArgs.push('--db-container', dbContainer);
  if (dryRun) scriptArgs.push('--dry-run');
  return {
    scriptName: 'apply-corpus-import-manifest.mjs',
    scriptArgs,
    env: { DEEPWELL_SESSION_TOKEN: sessionToken },
  };
}

// Merge the per-batch apply summaries (each the `summary` object printed by
// apply-corpus-import-manifest.mjs) into one additive summary.
export function mergeApplySummaries(summaries) {
  const merged = {};
  for (const summary of summaries) {
    for (const [key, value] of Object.entries(summary ?? {})) {
      if (typeof value !== 'number') continue;
      merged[key] = (merged[key] ?? 0) + value;
    }
  }
  return merged;
}

// Parse the stdout of one apply run: single-line JSON per-row action records,
// followed by a pretty-printed {"summary": {...}} object (or a pretty-printed
// {"dry_run": ...} object when --dry-run is used).
export function parseApplyOutput(stdout) {
  const rows = [];
  let summary = null;
  let block = null;
  for (const line of stdout.split('\n')) {
    if (block !== null) {
      block.push(line);
      try {
        const parsed = JSON.parse(block.join('\n'));
        block = null;
        if (parsed !== null && typeof parsed === 'object' && 'summary' in parsed) {
          summary = parsed.summary;
        } else if (parsed !== null && typeof parsed === 'object' && 'dry_run' in parsed) {
          summary = parsed;
        }
      } catch {
        // Block is still incomplete; keep accumulating lines.
      }
      continue;
    }
    const text = line.trim();
    if (!text.startsWith('{')) continue;
    try {
      const parsed = JSON.parse(text);
      if (parsed !== null && typeof parsed === 'object' && 'summary' in parsed) {
        summary = parsed.summary;
      } else {
        rows.push(parsed);
      }
    } catch {
      // Start of a pretty-printed multi-line object.
      block = [line];
    }
  }
  return { rows, summary };
}
