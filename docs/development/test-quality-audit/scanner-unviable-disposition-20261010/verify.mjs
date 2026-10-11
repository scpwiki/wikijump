#!/usr/bin/env node
// Cross-check complete historical rustc logs with the portable mutation outcomes.
// This is provenance verification, NOT a test of Wikidot/renderer behavior.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(readFileSync(resolve(root, 'manifest.json'), 'utf8'));
const ledger = resolve(root, manifest.frozen_ledger);
const first = JSON.parse(readFileSync(resolve(ledger, 'inputs/01-initial/outcomes.json'), 'utf8'));
const ledgerManifest = JSON.parse(readFileSync(resolve(ledger, 'manifest.json'), 'utf8'));
assert.equal(manifest.schema, 1);
assert.equal(manifest.entries.length, 4);
assert.equal(ledgerManifest.distinct_mutants, 136);
assert.deepEqual(ledgerManifest.reconciled, {caught: 132, missed: 0, unviable: 4, timeout: 0});
assert.equal(manifest.frozen_scanner_source_sha256, ledgerManifest.scanner_source_sha256);
const initialUnviable = new Set(first.outcomes.filter(x => x.summary === 'Unviable' && typeof x.scenario?.Mutant?.name === 'string').map(x => x.scenario.Mutant.name));
assert.equal(initialUnviable.size, 4);
const checked = new Set();
for (const row of manifest.entries) {
  assert(!checked.has(row.mutation));
  checked.add(row.mutation);
  assert(initialUnviable.has(row.mutation), `not frozen Unviable: ${row.mutation}`);
  assert(/^compiler-logs\/1426-return-default-\d{2}\.log\.gz$/.test(row.file));
  const compressed = readFileSync(resolve(root, row.file));
  assert.equal(createHash('sha256').update(compressed).digest('hex'), row.gzip_sha256);
  const bytes = gunzipSync(compressed);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), row.sha256);
  const log = bytes.toString('utf8');
  assert(log.includes(`*** ${row.mutation}\n`), 'mismatched cargo-mutants identity');
  assert(log.includes('error[E0277]'), 'missing rustc E0277 compiler failure');
  assert(log.includes('the trait bound `ListPagesModuleMatch<'), 'missing exact type diagnostic');
  assert(log.includes('Default` is not satisfied'), 'missing missing-Default diagnostic');
  assert(log.includes('could not compile `deepwell`'), 'missing failed compile');
}
assert.deepEqual(checked, initialUnviable);
console.log('Scanner Unviable compiler provenance PASS: 4/4 E0277 logs tied to frozen 136-ID ledger.');
