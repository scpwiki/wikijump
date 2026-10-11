#!/usr/bin/env node
// Verify portable, sealed scanner mutation outcomes with the maintained reconciler.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtempSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const bundle = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(bundle, '../../../..');
const manifest = JSON.parse(readFileSync(join(bundle, 'manifest.json'), 'utf8'));
const reconciler = join(repoRoot, 'scripts/reconcile-test-quality-mutants.mjs');
if (manifest.schema !== 1 || manifest.source_files.length !== 33) throw new Error('unexpected mutation ledger schema/count');
const args = [];
let seenReplay = false;
for (const source of manifest.source_files) {
  if (!['initial', 'replay'].includes(source.role)) throw new Error('invalid role');
  if (source.role === 'initial' && seenReplay) throw new Error('initial after replay');
  if (source.role === 'replay') seenReplay = true;
  const filepath = resolve(bundle, source.path);
  if (!filepath.startsWith(join(bundle, 'inputs') + '/')) throw new Error('input escapes bundle');
  const digest = createHash('sha256').update(readFileSync(filepath)).digest('hex');
  if (digest !== source.sha256) throw new Error(`receipt SHA mismatch: ${source.path}`);
  args.push(source.role === 'initial' ? '--initial' : '--replay', dirname(filepath));
}
const dir = mkdtempSync(join(tmpdir(), 'wj-scanner-ledger-'));
try {
  const report = join(dir, 'report.json');
  for (const arguments_ of [[...args, '--output', report], ['--verify-report', report]]) {
    const result = spawnSync(process.execPath, [reconciler, ...arguments_], {cwd: repoRoot, encoding: 'utf8'});
    process.stdout.write(result.stdout);
    process.stderr.write(result.stderr);
    if (result.status !== 0) throw new Error(`reconciliation command failed: ${result.status}`);
  }
  const result = JSON.parse(readFileSync(report, 'utf8'));
  if (result.frozen_mutants !== manifest.distinct_mutants ||
      JSON.stringify(result.reconciled) !== JSON.stringify(manifest.reconciled) ||
      JSON.stringify(result.unresolved_survivors) !== JSON.stringify(manifest.unresolved_survivors)) {
    throw new Error('reconciled aggregate or survivor identity differs from manifest');
  }
  process.stdout.write('Portable scanner mutation ledger PASS.\n');
} finally {
  rmSync(dir, {recursive: true, force: true});
}
