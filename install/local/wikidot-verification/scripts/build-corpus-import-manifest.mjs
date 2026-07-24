#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
} from '../src/corpus-import-manifest.mjs';
import { stableStringify } from '../src/canonical-json.mjs';

export function usage() {
  return 'Usage: build-corpus-import-manifest.mjs (--corpus-root <path> [--branch en] | --source-bundle <path>) [--source-site site] [--source-branch branch] [--fullname <page>...] --output <manifest.jsonl> --summary <summary.json>';
}

export function parseArgs(argv) {
  const args = {
    corpusRoot: null,
    sourceBundle: null,
    branch: 'en',
    sourceSite: null,
    sourceBranch: null,
    output: null,
    summary: null,
    fullnames: [],
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
    else if (arg === '--source-bundle') args.sourceBundle = next();
    else if (arg === '--branch') args.branch = next();
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--source-branch') args.sourceBranch = next();
    else if (arg === '--output') args.output = next();
    else if (arg === '--summary') args.summary = next();
    else if (arg === '--fullname') args.fullnames.push(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if ((args.corpusRoot === null) === (args.sourceBundle === null)) {
    throw new Error('exactly one of --corpus-root or --source-bundle is required');
  }
  if (!args.output) throw new Error('--output is required');
  if (!args.summary) throw new Error('--summary is required');
  if (args.corpusRoot !== null) {
    args.sourceSite ??= args.branch;
    args.sourceBranch ??= args.branch;
  }
  return args;
}

export function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const manifestInput = {
    corpusRoot: args.corpusRoot === null ? null : path.resolve(args.corpusRoot),
    sourceBundleRoot: args.sourceBundle === null ? null : path.resolve(args.sourceBundle),
    branch: args.branch,
    sourceSite: args.sourceSite,
  };
  if (args.sourceBranch !== null) manifestInput.sourceBranch = args.sourceBranch;
  if (args.fullnames.length > 0) manifestInput.fullnames = args.fullnames;
  const rows = buildCorpusImportManifest(manifestInput);
  const jsonl = formatJsonl(rows);
  const summary = buildManifestSummary(rows, jsonl);
  fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
  fs.mkdirSync(path.dirname(path.resolve(args.summary)), { recursive: true });
  fs.writeFileSync(args.output, jsonl);
  fs.writeFileSync(args.summary, `${stableStringify(summary)}\n`);
  console.log(stableStringify(summary));
  return 0;
}

await runCliIfMain(import.meta.url, main);
