#!/usr/bin/env node
// Dependency-closure report (agent-runnable, P2): resolve include / theme /
// parent / attachment dependencies for a set of pages against the registered
// source bundles and emit fail-closed verdicts. Exit codes: 0 all closures
// complete or all out-of-bundle deps classified, 1 unclassified out-of-bundle
// dependencies, 2 structural error.
//
// Usage:
//   dependency-closure-report.mjs --inventory <corpus-inventory.lock.json> \
//     --slug-file <slugs.txt> --output-dir <dir> [--family EN] [--max-depth 8]

import fs from 'node:fs';
import path from 'node:path';

import {runCliIfMain} from '../src/cli-entry.mjs';

import {
  buildBundleRegistry,
  resolveDependencyClosure,
  summarizeClosureReports,
} from '../src/dependency-closure.mjs';

export function parseArgs(argv) {
  const args = { inventory: null, slugFile: null, outputDir: null, family: null, maxDepth: 8 };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => argv[++i];
    if (arg === '--inventory') args.inventory = next();
    else if (arg === '--slug-file') args.slugFile = next();
    else if (arg === '--output-dir') args.outputDir = next();
    else if (arg === '--family') args.family = next();
    else if (arg === '--max-depth') args.maxDepth = Number(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.inventory) throw new Error('--inventory is required');
  if (!args.slugFile) throw new Error('--slug-file is required');
  if (!args.outputDir) throw new Error('--output-dir is required');
  return args;
}

function readSourceArtifact(row) {
  const sourcePath = row.source_artifact ?? row.source_path ?? null;
  if (!sourcePath) return null;
  try {
    return fs.readFileSync(sourcePath, 'utf8');
  } catch {
    return null;
  }
}

export function usage() {
  return 'Usage: dependency-closure-report.mjs --inventory <lock.json> --slug-file <slugs.txt> --output-dir <dir> [--family EN] [--max-depth 8]';
}

export function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const inventory = JSON.parse(fs.readFileSync(args.inventory, 'utf8'));
  const rows = inventory.rows ?? inventory;
  if (!Array.isArray(rows)) throw new Error('inventory has no rows array');

  const registry = buildBundleRegistry(rows);
  const slugs = fs
    .readFileSync(args.slugFile, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);

  const byKey = new Map();
  for (const row of rows) {
    if (args.family && row.family !== args.family) continue;
    const slug = row.slug ?? row.fullname;
    if (slug) byKey.set(slug, row);
    if (row.fixture_id) byKey.set(row.fixture_id, row);
  }

  fs.mkdirSync(path.join(args.outputDir, 'pages'), { recursive: true });
  const reports = [];
  const missing = [];
  for (const slug of slugs) {
    const row = byKey.get(slug);
    if (!row) {
      missing.push(slug);
      continue;
    }
    const report = resolveDependencyClosure({
      row,
      registry,
      readSource: readSourceArtifact,
      maxDepth: args.maxDepth,
    });
    reports.push(report);
    const fileName = `${report.fixture_id.replace(/[^a-zA-Z0-9_-]+/g, '_')}.json`;
    fs.writeFileSync(
      path.join(args.outputDir, 'pages', fileName),
      JSON.stringify(report, null, 1),
    );
  }

  const summary = summarizeClosureReports(reports);
  summary.slugs_not_in_inventory = missing;
  summary.registry_collisions = registry.collisions.length;
  if (registry.collisions.length > 0) {
    fs.writeFileSync(
      path.join(args.outputDir, 'registry-collisions.json'),
      JSON.stringify(registry.collisions, null, 1),
    );
  }
  fs.writeFileSync(
    path.join(args.outputDir, 'closure-summary.json'),
    JSON.stringify(summary, null, 1),
  );
  console.log(JSON.stringify(summary, null, 2));
  if (missing.length > 0) return 2;
  return summary.exit_code;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(String(error?.stack ?? error));
    return 2;
  },
});
