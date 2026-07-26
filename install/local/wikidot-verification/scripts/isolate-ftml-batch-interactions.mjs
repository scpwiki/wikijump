#!/usr/bin/env node

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  buildSavedPagePlans,
  isolateBatchInteractions,
  summarizeLiveCases,
} from '../src/ftml-live-cases.mjs';

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    cases: null,
    verification: null,
    casesOutput: null,
    pagesOutput: null,
    slugPrefix: null,
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--cases') args.cases = optionValue(argv, index++, arg);
    else if (arg === '--verification') args.verification = optionValue(argv, index++, arg);
    else if (arg === '--cases-output') args.casesOutput = optionValue(argv, index++, arg);
    else if (arg === '--pages-output') args.pagesOutput = optionValue(argv, index++, arg);
    else if (arg === '--slug-prefix') args.slugPrefix = optionValue(argv, index++, arg);
    else if (arg === '--target-characters') args.targetCharacters = Number(optionValue(argv, index++, arg));
    else if (arg === '--hard-characters') args.hardCharacters = Number(optionValue(argv, index++, arg));
    else throw new Error(`Unknown argument: ${arg}`);
  }
  for (const [name, value] of Object.entries(args).filter(([name]) => !name.endsWith('Characters'))) {
    if (!value) throw new Error(`--${name.replace(/[A-Z]/gu, (letter) => `-${letter.toLowerCase()}`)} is required`);
  }
  if (!Number.isSafeInteger(args.targetCharacters) || args.targetCharacters <= 0) {
    throw new Error('--target-characters must be a positive integer');
  }
  if (!Number.isSafeInteger(args.hardCharacters) || args.hardCharacters < args.targetCharacters) {
    throw new Error('--hard-characters must be an integer at least as large as --target-characters');
  }
  return args;
}

function readJsonLines(filePath) {
  return fs.readFileSync(filePath, 'utf8').split('\n').filter((line) => line.trim()).map(JSON.parse);
}

function writeJsonLines(filePath, values) {
  fs.writeFileSync(filePath, `${values.map((value) => JSON.stringify(value)).join('\n')}\n`, {flag: 'wx'});
}

export async function main(argv) {
  const args = parseArgs(argv);
  const verification = JSON.parse(fs.readFileSync(args.verification, 'utf8'));
  const cases = isolateBatchInteractions(readJsonLines(args.cases), verification);
  const pages = buildSavedPagePlans(cases, {
    slugPrefix: args.slugPrefix,
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
