#!/usr/bin/env node
// V3 golden-pair comparison (agent-runnable): compare local renders against
// frozen live Wikidot captures from the golden-pairs catalog.
//
// Modes:
//   frozen  — compare the frozen live capture against the frozen local capture
//             (equivalence proving against the historical validator verdicts).
//   records — compare the frozen live capture against a FRESH local capture
//             records.json produced by capture-browser-rendering.mjs.
//
// Usage:
//   compare-render-evidence.mjs --pairs <golden-pairs.catalog.json> \
//     --output-dir <dir> [--mode frozen|records] [--records <records.json>] \
//     [--ledger <accepted-diff-ledger.jsonl>] [--run-id <id>] \
//     [--channel <name>=on|off ...]
//
// Exit codes: 0 zero regressions, 1 regressions present, 2 structural failure.

import fs from 'node:fs';
import path from 'node:path';

import {runCliIfMain} from '../src/cli-entry.mjs';

import {
  aggregateCompareVerdict,
  comparePair,
  DEFAULT_CHANNELS,
} from '../src/render-compare.mjs';

export function usage() {
  return 'Usage: compare-render-evidence.mjs --pairs <catalog.json> --output-dir <dir> ' +
    '[--mode frozen|records] [--records <records.json>] [--ledger <ledger.jsonl>] ' +
    '[--run-id id] [--channel name=on|off ...]';
}

export function parseArgs(argv) {
  const args = {
    pairs: null,
    outputDir: null,
    mode: 'frozen',
    records: null,
    ledger: null,
    runId: `v3-${new Date().toISOString().replace(/[:.]/g, '-')}`,
    channels: {},
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => argv[++i];
    if (arg === '--pairs') args.pairs = next();
    else if (arg === '--output-dir') args.outputDir = next();
    else if (arg === '--mode') args.mode = next();
    else if (arg === '--records') args.records = next();
    else if (arg === '--ledger') args.ledger = next();
    else if (arg === '--run-id') args.runId = next();
    else if (arg === '--channel') {
      const [name, state] = next().split('=');
      if (!(name in DEFAULT_CHANNELS)) throw new Error(`Unknown normalization channel: ${name}`);
      args.channels[name] = state !== 'off';
    } else if (arg === '--help' || arg === '-h') {
      return {help: true};
    } else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.pairs) throw new Error('--pairs is required');
  if (!args.outputDir) throw new Error('--output-dir is required');
  if (args.mode === 'records' && !args.records) throw new Error('--mode records requires --records');
  return args;
}

function loadLedger(ledgerPath) {
  if (!ledgerPath) return [];
  return fs
    .readFileSync(ledgerPath, 'utf8')
    .trim()
    .split('\n')
    .filter(Boolean)
    .map((line) => JSON.parse(line));
}

// Extract the evidence record for a fixture from a
// wikijump_full_parity.browser_rendering_evidence.v1 records.json.
function recordsByFixture(recordsJson) {
  const byFixture = new Map();
  for (const record of recordsJson.evidence ?? []) {
    if (record.fixture_id) byFixture.set(record.fixture_id, record);
  }
  return byFixture;
}

function frozenRecordFor(pair, catalogDir) {
  // evidence_directory may be relative to the catalog file (CI subset) or absolute.
  const evidenceDir = pair.evidence_directory
    ? path.resolve(catalogDir, pair.evidence_directory)
    : null;
  const recordsPath =
    pair.artifacts?.find((a) => a.name === 'records.json')?.dest_path ??
    (evidenceDir ? path.join(evidenceDir, 'records.json') : null);
  if (!recordsPath || !fs.existsSync(recordsPath)) return null;
  const records = JSON.parse(fs.readFileSync(recordsPath, 'utf8'));
  return recordsByFixture(records).get(pair.fixture_id) ?? null;
}

export function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const catalog = JSON.parse(fs.readFileSync(args.pairs, 'utf8'));
  const ledger = loadLedger(args.ledger);
  const freshRecords =
    args.mode === 'records'
      ? recordsByFixture(JSON.parse(fs.readFileSync(args.records, 'utf8')))
      : null;

  const pairs = [];
  const skipped = [];
  const catalogDir = path.dirname(path.resolve(args.pairs));
  for (const pair of catalog.pairs ?? []) {
    const frozen = frozenRecordFor(pair, catalogDir);
    if (!frozen) {
      skipped.push({ fixture_id: pair.fixture_id, reason: 'frozen records.json missing or fixture absent' });
      continue;
    }
    let localVisibleText = frozen.local_visible_text;
    let localUrl = frozen.local_url;
    let localArtifact = frozen.local_browser_artifact;
    if (freshRecords) {
      const fresh = freshRecords.get(pair.fixture_id);
      if (!fresh) {
        skipped.push({ fixture_id: pair.fixture_id, reason: 'fixture missing from fresh records' });
        continue;
      }
      localVisibleText = fresh.local_visible_text;
      localUrl = fresh.local_url;
      localArtifact = fresh.local_browser_artifact;
    }
    pairs.push(
      comparePair({
        fixtureId: pair.fixture_id,
        sourceVisibleText: frozen.source_visible_text,
        localVisibleText,
        sourceUrl: frozen.source_url,
        localUrl,
        sourceArtifact: frozen.source_browser_artifact,
        localArtifact,
        channels: args.channels,
        ledger,
      }),
    );
  }

  const { verdict, exitCode } = aggregateCompareVerdict({ runId: args.runId, pairs });
  verdict.mode = args.mode;
  verdict.skipped = skipped;

  fs.mkdirSync(args.outputDir, { recursive: true });
  fs.writeFileSync(path.join(args.outputDir, 'verdict.json'), JSON.stringify(verdict, null, 1));
  console.log(
    JSON.stringify(
      { run_id: args.runId, mode: args.mode, ...verdict.aggregate, skipped: skipped.length, exit_code: exitCode },
      null,
      2,
    ),
  );
  // Skipped pairs are structural: the catalog promised evidence we could not read.
  return skipped.length > 0 ? 2 : exitCode;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
