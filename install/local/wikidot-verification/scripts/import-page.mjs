#!/usr/bin/env node
// Single-page on-demand import runner (operator-only local tool).
//
// Given one or more corpus page slugs, makes them viewable on the local
// Wikijump mirror in one command: verifies the sources exist in the corpus
// (fail-closed; never fetches from live Wikidot), resolves the dependency
// closure against the corpus inventory, imports the missing pages plus
// dependencies via apply-corpus-import-manifest.mjs (attachments included),
// and runs the V2 render-health check on the imported pages.
//
// Usage:
//   import-page.mjs --slug <page> [--slug <page>...] \
//     --corpus-root <path> --output-dir <dir> \
//     [--inventory <corpus-inventory.lock.json>] [--branch en] [--family EN] \
//     [--site scp-wiki] [--site-id <id>] [--source-site <site>] [--source-branch <branch>] \
//     [--host <site>.wikijump.localhost] [--rpc-url http://127.0.0.1:2747/jsonrpc] \
//     [--db-container <name>] [--attachment-user-id -1] \
//     [--batch-size 40] [--max-depth 8] [--adopt-existing] [--skip-health] [--dry-run]
//
// Fail-closed rules: requested slugs missing from the corpus abort the run
// (exit 2); dependency closure exit codes 1 (unclassified out-of-bundle) and
// 2 (structural) abort the run; out-of-bundle dependencies are reported, not
// imported. Exit 0 means every requested page imported (or already present)
// and the health check ran; import failures exit 1.
//
// Auth: set DEEPWELL_SESSION_TOKEN, or the runner logs in
// per batch via the local Deepwell RPC using WIKIDOT_VERIFY_ADMIN_EMAIL /
// WIKIDOT_VERIFY_ADMIN_PASS (defaults match the local seed admin, same as
// preview-source.mjs). Tokens expire after ~5 minutes, hence per-batch login.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

import {
  batchSlugs,
  buildApplyInvocation,
  corpusPageStatus,
  mergeApplySummaries,
  parseApplyOutput,
  planImportSet,
  resolveSessionToken,
  rpcCall,
  validateRpcUrl,
} from '../src/import-page.mjs';

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));

function parseArgs(argv) {
  const args = {
    slugs: [],
    corpusRoot: process.env.WIKIDOT_VERIFY_CORPUS_ROOT ?? null,
    inventory: process.env.WIKIDOT_VERIFY_INVENTORY ?? null,
    outputDir: null,
    branch: 'en',
    family: null,
    site: 'scp-wiki',
    siteId: null,
    sourceSite: null,
    sourceBranch: null,
    host: null,
    rpcUrl: process.env.WIKIDOT_VERIFY_RPC_URL ?? 'http://127.0.0.1:2747/jsonrpc',
    sessionToken: process.env.DEEPWELL_SESSION_TOKEN ?? null,
    dbContainer: null,
    attachmentUserId: '-1',
    batchSize: 40,
    maxDepth: 8,
    adoptExisting: false,
    skipHealth: false,
    dryRun: false,
  };
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => {
      i += 1;
      if (i >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[i];
    };
    if (arg === '--slug') args.slugs.push(next());
    else if (arg === '--corpus-root') args.corpusRoot = next();
    else if (arg === '--inventory') args.inventory = next();
    else if (arg === '--output-dir') args.outputDir = next();
    else if (arg === '--branch') args.branch = next();
    else if (arg === '--family') args.family = next();
    else if (arg === '--site') args.site = next();
    else if (arg === '--site-id') args.siteId = Number.parseInt(next(), 10);
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--source-branch') args.sourceBranch = next();
    else if (arg === '--host') args.host = next();
    else if (arg === '--rpc-url') args.rpcUrl = next();
    else if (arg === '--db-container') args.dbContainer = next();
    else if (arg === '--attachment-user-id') args.attachmentUserId = next();
    else if (arg === '--batch-size') args.batchSize = Number.parseInt(next(), 10);
    else if (arg === '--max-depth') args.maxDepth = Number.parseInt(next(), 10);
    else if (arg === '--adopt-existing') args.adoptExisting = true;
    else if (arg === '--skip-health') args.skipHealth = true;
    else if (arg === '--dry-run') args.dryRun = true;
    else if (arg === '--help' || arg === '-h') {
      console.log(
        'Usage: import-page.mjs --slug <page> [--slug <page>...] --corpus-root <path> --output-dir <dir> ' +
          '[--inventory <lock.json>] [--branch en] [--family EN] [--site scp-wiki] [--site-id <id>] ' +
          '[--source-site <site>] [--source-branch <branch>] [--host <domain>] [--rpc-url <url>] ' +
          '[--db-container <name>] [--attachment-user-id -1] ' +
          '[--batch-size 40] [--max-depth 8] [--adopt-existing] [--skip-health] [--dry-run]',
      );
      process.exit(0);
    } else if (!arg.startsWith('--')) args.slugs.push(arg);
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.slugs.length === 0) throw new Error('at least one --slug is required');
  if (!args.corpusRoot) throw new Error('--corpus-root is required (or WIKIDOT_VERIFY_CORPUS_ROOT)');
  if (!args.outputDir) throw new Error('--output-dir is required');
  args.sourceSite ??= args.site;
  args.sourceBranch ??= args.branch;
  args.family ??= args.branch.toUpperCase();
  args.host ??= `${args.site}.wikijump.localhost`;
  const rpc = validateRpcUrl(args.rpcUrl);
  args.rpcUrl = rpc.rpcUrl;
  for (const warning of rpc.warnings) console.error(`WARN: ${warning}`);
  return args;
}

function runNode(scriptName, scriptArgs, { env = {}, logPath = null } = {}) {
  const result = spawnSync(
    process.execPath,
    [path.join(SCRIPT_DIR, scriptName), ...scriptArgs],
    { encoding: 'utf8', env: { ...process.env, ...env }, maxBuffer: 64 * 1024 * 1024 },
  );
  if (logPath !== null) {
    fs.writeFileSync(logPath, `${result.stdout ?? ''}\n--- stderr ---\n${result.stderr ?? ''}`);
  }
  if (result.error) throw result.error;
  return result;
}

async function loginToken(rpcUrl) {
  const result = await rpcCall(rpcUrl, 'login', {
    name_or_email: process.env.WIKIDOT_VERIFY_ADMIN_EMAIL ?? 'admin@wikijump',
    password: process.env.WIKIDOT_VERIFY_ADMIN_PASS ?? 'wikijumpadmin1',
    ip_address: '127.0.0.1',
    user_agent: 'import-page/0.1',
  });
  return result.session_token;
}

function fail(code, payload) {
  console.error(JSON.stringify(payload, null, 2));
  process.exit(code);
}

async function main() {
  const args = parseArgs(process.argv);
  fs.mkdirSync(args.outputDir, { recursive: true });
  const out = (...parts) => path.join(args.outputDir, ...parts);

  // 1. Fail closed on requested slugs missing from the corpus. Live Wikidot
  // is never consulted; pages outside the corpus are reported and the run
  // stops before mutating anything.
  const corpusProblems = {};
  for (const slug of args.slugs) {
    const status = corpusPageStatus(args.corpusRoot, args.branch, slug);
    if (status.status !== 'ok') corpusProblems[slug] = status;
  }
  if (Object.keys(corpusProblems).length > 0) {
    fail(2, { error: 'requested pages missing from corpus', corpus_root: args.corpusRoot, branch: args.branch, pages: corpusProblems });
  }

  // 2. Dependency closure (optional but recommended). Without --inventory the
  // requested pages import alone and unresolved includes surface in V2.
  let plan = planImportSet({ requestedSlugs: args.slugs });
  let closureSummary = null;
  if (args.inventory) {
    const slugFile = out('requested-slugs.txt');
    fs.writeFileSync(slugFile, `${args.slugs.join('\n')}\n`);
    const closureDir = out('closure');
    const closure = runNode('dependency-closure-report.mjs', [
      '--inventory', args.inventory,
      '--slug-file', slugFile,
      '--output-dir', closureDir,
      '--family', args.family,
      '--max-depth', String(args.maxDepth),
    ], { logPath: out('closure.log') });
    closureSummary = JSON.parse(fs.readFileSync(path.join(closureDir, 'closure-summary.json'), 'utf8'));
    if (closure.status !== 0) {
      fail(closure.status === 1 ? 1 : 2, {
        error: 'dependency closure failed closed',
        exit_code: closure.status,
        summary: closureSummary,
        log: out('closure.log'),
      });
    }
    const reports = fs.readdirSync(path.join(closureDir, 'pages'))
      .map((name) => JSON.parse(fs.readFileSync(path.join(closureDir, 'pages', name), 'utf8')));
    plan = planImportSet({ requestedSlugs: args.slugs, closureReports: reports });
  } else {
    console.error('WARN: no --inventory given; importing without dependency closure');
  }

  // Dependencies present in the inventory but not in this corpus checkout are
  // skipped (recorded), so one damaged dependency cannot block the page.
  const dependencySlugs = [];
  const skippedDependencies = {};
  for (const slug of plan.dependencySlugs) {
    const status = corpusPageStatus(args.corpusRoot, args.branch, slug);
    if (status.status === 'ok') dependencySlugs.push(slug);
    else skippedDependencies[slug] = status;
  }
  const importSlugs = [...dependencySlugs, ...args.slugs];

  // 3. Build the manifest for exactly the import set.
  const manifestPath = out('manifest.jsonl');
  const build = runNode('build-corpus-import-manifest.mjs', [
    '--corpus-root', args.corpusRoot,
    '--branch', args.branch,
    '--source-site', args.sourceSite,
    '--source-branch', args.sourceBranch,
    ...importSlugs.flatMap((slug) => ['--fullname', slug]),
    '--output', manifestPath,
    '--summary', out('manifest-summary.json'),
  ], { logPath: out('build-manifest.log') });
  if (build.status !== 0) {
    fail(2, { error: 'manifest build failed', log: out('build-manifest.log') });
  }

  // 4. Resolve the target site id explicitly; the apply default is the
  // template site, which is a known mis-import trap.
  let siteId = args.siteId;
  if (siteId === null) {
    const site = await rpcCall(args.rpcUrl, 'site_get', { site: args.site });
    if (!site || !Number.isInteger(site.site_id)) {
      fail(2, { error: `site_get did not resolve site '${args.site}'` });
    }
    siteId = site.site_id;
  }

  // 5. Apply in batches with a fresh login per batch (~5 minute token TTL).
  const manifestLines = fs.readFileSync(manifestPath, 'utf8').split('\n').filter(Boolean);
  const rowBySlug = new Map(manifestLines.map((line) => [JSON.parse(line).fullname, line]));
  const batches = batchSlugs(importSlugs, args.batchSize);
  const applyResults = [];
  const applySummaries = [];
  for (const [index, batch] of batches.entries()) {
    const batchPath = out(`apply-batch-${index}.jsonl`);
    fs.writeFileSync(batchPath, `${batch.map((slug) => rowBySlug.get(slug)).join('\n')}\n`);
    const sessionToken = await resolveSessionToken({
      sessionToken: args.sessionToken,
      rpcUrl: args.rpcUrl,
      login: loginToken,
    });
    const invocation = buildApplyInvocation({
      batchPath,
      rpcUrl: args.rpcUrl,
      siteId,
      attachmentUserId: args.attachmentUserId,
      sessionToken,
      adoptExisting: args.adoptExisting,
      dbContainer: args.dbContainer,
      dryRun: args.dryRun,
    });
    const apply = runNode(invocation.scriptName, invocation.scriptArgs, {
      env: invocation.env,
      logPath: out(`apply-batch-${index}.log`),
    });
    const parsed = parseApplyOutput(apply.stdout ?? '', {requireTerminal: apply.status === 0});
    applyResults.push(...parsed.rows);
    if (parsed.summary) applySummaries.push(parsed.summary);
    if (apply.status !== 0) {
      fail(1, {
        error: 'apply failed',
        batch: index,
        log: out(`apply-batch-${index}.log`),
        rows: parsed.rows.filter((row) => String(row.action ?? '').startsWith('collision') || row.action === 'failed' || row.action === 'render_failed'),
      });
    }
  }

  // 6. V2 render-health over the imported set.
  let healthVerdict = null;
  if (!args.skipHealth && !args.dryRun) {
    const health = runNode('render-health-sweep.mjs', [
      '--manifest', manifestPath,
      '--host', args.host,
      '--output-dir', out('health'),
      '--family', args.family,
      '--insecure-local-tls',
    ], { logPath: out('health.log') });
    const verdictPath = path.join(out('health'), 'verdict.json');
    if (fs.existsSync(verdictPath)) {
      healthVerdict = JSON.parse(fs.readFileSync(verdictPath, 'utf8'));
    }
    if (health.status === 2 || healthVerdict === null) {
      fail(2, { error: 'render-health sweep failed structurally', log: out('health.log') });
    }
  }

  const actionBySlug = Object.fromEntries(applyResults.map((row) => [row.slug, row.action]));
  const summary = {
    requested: args.slugs,
    site_id: siteId,
    urls: Object.fromEntries(args.slugs.map((slug) => [slug, `https://${args.host}/${slug}`])),
    dependencies_imported: dependencySlugs,
    dependencies_skipped: skippedDependencies,
    out_of_bundle_dependencies: plan.outOfBundle,
    closure: closureSummary,
    apply: mergeApplySummaries(applySummaries),
    actions: actionBySlug,
    health: healthVerdict?.pages ?? healthVerdict,
  };
  fs.writeFileSync(out('import-page-summary.json'), `${JSON.stringify(summary, null, 2)}\n`);
  console.log(JSON.stringify(summary, null, 2));
  for (const slug of args.slugs) {
    console.error(`${slug}: ${actionBySlug[slug] ?? 'no-action'} -> https://${args.host}/${slug}`);
  }
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(2);
});
