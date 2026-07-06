import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const cli = path.join(here, '..', 'scripts', 'compare-render-evidence.mjs');
const ciCatalog = path.join(here, '..', 'fixtures', 'golden-pairs-ci', 'catalog.json');

function runCli(extraArgs, outputDir) {
  return spawnSync(process.execPath, [cli, '--pairs', ciCatalog, '--output-dir', outputDir, ...extraArgs], {
    encoding: 'utf8',
  });
}

test('V3 CLI reproduces expected verdicts on the CI golden-pair subset', () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), 'v3-ci-'));
  const result = runCli(['--run-id', 'ci-subset'], outputDir);
  // The subset intentionally includes one known-regression pair, so exit is 1
  // (not 2 — no structural failures).
  assert.equal(result.status, 1, result.stderr);
  const verdict = JSON.parse(fs.readFileSync(path.join(outputDir, 'verdict.json'), 'utf8'));
  const expected = JSON.parse(fs.readFileSync(ciCatalog, 'utf8'));
  assert.equal(verdict.pairs.length, expected.pairs.length);
  for (const pair of expected.pairs) {
    const got = verdict.pairs.find((p) => p.fixture_id === pair.fixture_id);
    assert.equal(got.verdict, pair.expected_verdict, pair.fixture_id);
  }
  fs.rmSync(outputDir, { recursive: true, force: true });
});

test('V3 CLI accepts a ledger that converts the known regression', () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), 'v3-ci-'));
  const expected = JSON.parse(fs.readFileSync(ciCatalog, 'utf8'));
  const regression = expected.pairs.find((p) => p.expected_verdict === 'regression');
  const ledgerPath = path.join(outputDir, 'ledger.jsonl');
  fs.writeFileSync(
    ledgerPath,
    JSON.stringify({
      category: 'visible_text_difference',
      scope: 'ci-subset-test',
      policy_reason: 'test-only ledger entry',
      fixture_ids: [regression.fixture_id],
    }) + '\n',
  );
  const result = runCli(['--run-id', 'ci-subset-ledger', '--ledger', ledgerPath], outputDir);
  const verdict = JSON.parse(fs.readFileSync(path.join(outputDir, 'verdict.json'), 'utf8'));
  const got = verdict.pairs.find((p) => p.fixture_id === regression.fixture_id);
  assert.equal(got.verdict, 'accepted-diff');
  assert.equal(result.status, 0, result.stderr);
  fs.rmSync(outputDir, { recursive: true, force: true });
});
