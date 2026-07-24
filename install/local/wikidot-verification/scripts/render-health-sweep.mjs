#!/usr/bin/env node
// V2 render-health sweep (agent-runnable): fetch locally rendered pages and
// classify them per the render-health taxonomy spec. Produces verdict.json and
// lab-health-dashboard.html in --output-dir; exit code follows the spec
// (0 pass, 1 below threshold, 2 taxonomy-unknown/structural).
//
// Usage:
//   render-health-sweep.mjs --manifest <manifest.jsonl> --host <site-host> \
//     --output-dir <dir> [--run-id <id>] [--family EN] [--threshold 0.9] \
//     [--concurrency 8] [--http-port 443] [--previous <verdict.json>] \
//     [--import-summary <apply-log.json>] [--disposition <category>=<disposition>...]
//
// The manifest is the corpus-import manifest (JSONL with fullname + source
// paths); source text is read to discount @@-escaped display markers.

import fs from 'node:fs';
import path from 'node:path';
import https from 'node:https';

import {runCliIfMain} from '../src/cli-entry.mjs';

import {
  aggregateVerdict,
  classifyRenderedPage,
  renderDashboardHtml,
} from '../src/render-health.mjs';

export function usage() {
  return 'Usage: render-health-sweep.mjs --manifest <manifest.jsonl> --host <site-host> --output-dir <dir> ' +
    '[--run-id id] [--family EN] [--threshold 0.9] [--concurrency 8] [--previous verdict.json] ' +
    '[--import-summary summary.json] [--disposition category=disposition ...]';
}

export function parseArgs(argv) {
  const args = {
    manifest: null,
    host: null,
    outputDir: null,
    runId: `v2-${new Date().toISOString().replace(/[:.]/g, '-')}`,
    family: 'EN',
    threshold: null,
    concurrency: 8,
    address: '127.0.0.1',
    port: 443,
    previous: null,
    importSummary: null,
    dispositions: {},
    insecureLocalTls: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => argv[++i];
    if (arg === '--manifest') args.manifest = next();
    else if (arg === '--host') args.host = next();
    else if (arg === '--output-dir') args.outputDir = next();
    else if (arg === '--run-id') args.runId = next();
    else if (arg === '--family') args.family = next();
    else if (arg === '--threshold') args.threshold = Number(next());
    else if (arg === '--concurrency') args.concurrency = Number(next());
    else if (arg === '--address') args.address = next();
    else if (arg === '--http-port') args.port = Number(next());
    else if (arg === '--previous') args.previous = next();
    else if (arg === '--insecure-local-tls') args.insecureLocalTls = true;
    else if (arg === '--import-summary') args.importSummary = next();
    else if (arg === '--disposition') {
      const [category, disposition] = next().split('=');
      args.dispositions[category] = disposition;
    } else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.manifest) throw new Error('--manifest is required');
  if (!args.host) throw new Error('--host is required');
  if (!args.outputDir) throw new Error('--output-dir is required');
  return args;
}

function fetchPage(args, slug, redirectsLeft = 5, pathOverride = null) {
  return new Promise((resolve) => {
    const req = https.request(
      {
        host: args.address,
        port: args.port,
        path: pathOverride ?? `/${encodeURI(slug)}`,
        headers: { Host: args.host },
        servername: args.host,
        // TLS verification stays on by default; --insecure-local-tls opts in
        // for the lab runtime's self-signed caddy CA on 127.0.0.1 only.
        rejectUnauthorized: !(args.insecureLocalTls && args.address === '127.0.0.1'),
        timeout: 30000,
      },
      (res) => {
        // Follow same-host redirects (e.g. platform domain -> canonical domain).
        if ([301, 302, 307, 308].includes(res.statusCode) && res.headers.location) {
          res.resume();
          if (redirectsLeft <= 0) {
            resolve({ status: res.statusCode, body: '', error: 'too many redirects' });
            return;
          }
          const location = new URL(res.headers.location, `https://${args.host}`);
          resolve(
            fetchPage(
              { ...args, host: location.host },
              slug,
              redirectsLeft - 1,
              `${location.pathname}${location.search}`,
            ),
          );
          return;
        }
        let body = '';
        res.on('data', (chunk) => {
          body += chunk;
        });
        res.on('end', () => resolve({ status: res.statusCode, body }));
      },
    );
    req.on('timeout', () => {
      req.destroy();
      resolve({ status: 0, body: '', error: 'timeout' });
    });
    req.on('error', (error) => resolve({ status: 0, body: '', error: String(error) }));
    req.end();
  });
}

function readSource(row) {
  const sourcePath = row.source_path ?? row.sourcePath ?? null;
  if (!sourcePath) return null;
  try {
    return fs.readFileSync(sourcePath, 'utf8');
  } catch {
    return null;
  }
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const rows = fs
    .readFileSync(args.manifest, 'utf8')
    .trim()
    .split('\n')
    .map((line) => JSON.parse(line));

  const pages = [];
  const queue = [...rows];
  async function worker() {
    for (;;) {
      const row = queue.shift();
      if (!row) return;
      const slug = row.fullname;
      const response = await fetchPage(args, slug);
      pages.push(
        classifyRenderedPage({
          fixtureId: `${args.family}:${slug}`,
          httpStatus: response.status ?? 0,
          html: response.body,
          source: readSource(row),
          dispositions: args.dispositions,
        }),
      );
      if (pages.length % 50 === 0) console.error(`${pages.length}/${rows.length}`);
    }
  }
  await Promise.all(Array.from({ length: args.concurrency }, worker));
  pages.sort((a, b) => a.fixture_id.localeCompare(b.fixture_id));

  const previous = args.previous ? JSON.parse(fs.readFileSync(args.previous, 'utf8')) : null;
  const importSummary = args.importSummary
    ? JSON.parse(fs.readFileSync(args.importSummary, 'utf8'))
    : null;

  const { verdict, exitCode } = aggregateVerdict({
    runId: args.runId,
    family: args.family,
    pages,
    threshold: args.threshold,
  });

  fs.mkdirSync(args.outputDir, { recursive: true });
  fs.writeFileSync(path.join(args.outputDir, 'verdict.json'), JSON.stringify(verdict, null, 1));
  fs.writeFileSync(
    path.join(args.outputDir, 'lab-health-dashboard.html'),
    renderDashboardHtml({ verdict, importSummary, previous }),
  );
  console.log(JSON.stringify({ ...verdict.aggregate, run_id: verdict.run_id, exit_code: exitCode }, null, 2));
  return exitCode;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
