#!/usr/bin/env node

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  buildSavedPagePlans,
  collectFtmlRecordedCases,
  summarizeLiveCases,
} from '../src/ftml-live-cases.mjs';

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    records: [],
    casesOutput: null,
    pagesOutput: null,
    slugPrefix: null,
    executionClass: 'saved-page-batch',
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--records') args.records.push(optionValue(argv, index++, arg));
    else if (arg === '--cases-output') args.casesOutput = optionValue(argv, index++, arg);
    else if (arg === '--pages-output') args.pagesOutput = optionValue(argv, index++, arg);
    else if (arg === '--slug-prefix') args.slugPrefix = optionValue(argv, index++, arg);
    else if (arg === '--execution-class') args.executionClass = optionValue(argv, index++, arg);
    else if (arg === '--target-characters') args.targetCharacters = Number(optionValue(argv, index++, arg));
    else if (arg === '--hard-characters') args.hardCharacters = Number(optionValue(argv, index++, arg));
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.records.length === 0) throw new Error('--records is required');
  for (const name of ['casesOutput', 'pagesOutput', 'slugPrefix']) {
    if (!args[name]) throw new Error(`--${name.replace(/[A-Z]/gu, (letter) => `-${letter.toLowerCase()}`)} is required`);
  }
  if (!Number.isSafeInteger(args.targetCharacters) || args.targetCharacters <= 0) {
    throw new Error('--target-characters must be a positive integer');
  }
  if (!Number.isSafeInteger(args.hardCharacters) || args.hardCharacters < args.targetCharacters) {
    throw new Error('--hard-characters must be an integer at least as large as --target-characters');
  }
  if (!['saved-page-batch', 'page-preview-isolated', 'wikijump-runtime'].includes(args.executionClass)) {
    throw new Error('--execution-class is unsupported');
  }
  return args;
}

function writeJsonLines(outputPath, values) {
  fs.writeFileSync(outputPath, `${values.map((value) => JSON.stringify(value)).join('\n')}\n`, {flag: 'wx'});
}

export async function main(argv) {
  const args = parseArgs(argv);
  const cases = collectFtmlRecordedCases(args.records);
  const pages = buildSavedPagePlans(cases, {
    slugPrefix: args.slugPrefix,
    executionClass: args.executionClass,
    targetCharacters: args.targetCharacters,
    hardCharacters: args.hardCharacters,
  });
  writeJsonLines(args.casesOutput, cases);
  writeJsonLines(args.pagesOutput, pages);
  console.log(JSON.stringify(summarizeLiveCases(cases, pages)));
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
