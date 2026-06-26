#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
  stableStringify,
} from '../src/corpus-import-manifest.mjs';

function parseArgs(argv) {
  const args = {
    corpusRoot: null,
    branch: 'en',
    sourceSite: null,
    sourceBranch: null,
    output: null,
    summary: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) {
        throw new Error(`${arg} requires a value`);
      }
      return argv[index];
    };
    if (arg === '--corpus-root') args.corpusRoot = next();
    else if (arg === '--branch') args.branch = next();
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--source-branch') args.sourceBranch = next();
    else if (arg === '--output') args.output = next();
    else if (arg === '--summary') args.summary = next();
    else if (arg === '--help' || arg === '-h') {
      console.log('Usage: build-corpus-import-manifest.mjs --corpus-root <path> [--branch en] --output <manifest.jsonl> --summary <summary.json>');
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!args.corpusRoot) throw new Error('--corpus-root is required');
  if (!args.output) throw new Error('--output is required');
  if (!args.summary) throw new Error('--summary is required');
  args.sourceSite ??= args.branch;
  args.sourceBranch ??= args.branch;
  return args;
}

const args = parseArgs(process.argv.slice(2));
const rows = buildCorpusImportManifest({
  corpusRoot: path.resolve(args.corpusRoot),
  branch: args.branch,
  sourceSite: args.sourceSite,
  sourceBranch: args.sourceBranch,
});
const jsonl = formatJsonl(rows);
const summary = buildManifestSummary(rows, jsonl);
fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
fs.mkdirSync(path.dirname(path.resolve(args.summary)), { recursive: true });
fs.writeFileSync(args.output, jsonl);
fs.writeFileSync(args.summary, `${stableStringify(summary)}\n`);
console.log(stableStringify(summary));
