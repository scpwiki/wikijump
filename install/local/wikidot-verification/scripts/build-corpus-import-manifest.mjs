#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
  summarizeIncludeClosure,
} from '../src/corpus-import-manifest.mjs';
import {
  DEFAULT_CLOSURE_MAX_DEPTH,
  DEFAULT_CLOSURE_MAX_PAGES,
  expandIncludeClosure,
} from '../src/corpus-include-closure.mjs';
import { codePointCompare, stableStringify } from '../src/canonical-json.mjs';

export function usage() {
  return 'Usage: build-corpus-import-manifest.mjs (--corpus-root <path> [--branch en] | --source-bundle <path>) [--source-site site] [--source-branch branch] [--fullname <page>...] [--include-closure [--closure-max-depth <n>] [--closure-max-pages <n>] [--allow-unresolved-closure]] --output <manifest.jsonl> --summary <summary.json>';
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
    includeClosure: false,
    closureMaxDepth: DEFAULT_CLOSURE_MAX_DEPTH,
    closureMaxPages: DEFAULT_CLOSURE_MAX_PAGES,
    allowUnresolvedClosure: false,
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
    else if (arg === '--include-closure') args.includeClosure = true;
    else if (arg === '--closure-max-depth') args.closureMaxDepth = parseIntegerOption(next(), '--closure-max-depth');
    else if (arg === '--closure-max-pages') args.closureMaxPages = parseIntegerOption(next(), '--closure-max-pages');
    else if (arg === '--allow-unresolved-closure') args.allowUnresolvedClosure = true;
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
  if (args.includeClosure && args.sourceBundle !== null) {
    throw new Error('--include-closure requires --corpus-root');
  }
  if (args.includeClosure && args.fullnames.length === 0) {
    throw new Error('--include-closure requires at least one --fullname root');
  }
  if ((args.closureMaxDepth !== DEFAULT_CLOSURE_MAX_DEPTH || args.closureMaxPages !== DEFAULT_CLOSURE_MAX_PAGES || args.allowUnresolvedClosure) && !args.includeClosure) {
    throw new Error('--closure-max-depth, --closure-max-pages, and --allow-unresolved-closure require --include-closure');
  }
  if (args.corpusRoot !== null) {
    args.sourceSite ??= args.branch;
    args.sourceBranch ??= args.branch;
  }
  return args;
}

export function parseIntegerOption(value, label) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed)) throw new Error(`${label} must be an integer`);
  return parsed;
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
  if (args.includeClosure) {
    manifestInput.includeClosure = {
      maxDepth: args.closureMaxDepth,
      maxPages: args.closureMaxPages,
      allowUnresolved: args.allowUnresolvedClosure,
    };
  }
  const rows = buildCorpusImportManifest(manifestInput);
  const jsonl = formatJsonl(rows);
  const summary = buildManifestSummary(rows, jsonl);
  if (args.includeClosure) {
    const pagesRoot = path.join(manifestInput.corpusRoot, args.branch, 'pages');
    const closure = expandIncludeClosure({
      roots: [...args.fullnames].sort(codePointCompare),
      readSource: (slug) => {
        const sourcePath = path.join(pagesRoot, slug, 'source.wikidot.txt');
        return fs.existsSync(sourcePath) ? fs.readFileSync(sourcePath, 'utf8') : null;
      },
      pageExists: (slug) => fs.existsSync(path.join(pagesRoot, slug, 'source.wikidot.txt')),
      sourceSite: args.sourceSite,
      maxDepth: args.closureMaxDepth,
      maxPages: args.closureMaxPages,
    });
    summary.include_closure = summarizeIncludeClosure(closure);
  }
  fs.mkdirSync(path.dirname(path.resolve(args.output)), { recursive: true });
  fs.mkdirSync(path.dirname(path.resolve(args.summary)), { recursive: true });
  fs.writeFileSync(args.output, jsonl);
  fs.writeFileSync(args.summary, `${stableStringify(summary)}\n`);
  console.log(stableStringify(summary));
  return 0;
}

await runCliIfMain(import.meta.url, main);
