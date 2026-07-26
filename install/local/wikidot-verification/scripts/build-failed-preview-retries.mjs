#!/usr/bin/env node

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {buildFailedPreviewRetryPlans} from '../src/ftml-live-cases.mjs';

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    cases: null,
    captures: null,
    output: null,
    slugPrefix: null,
    executionClass: null,
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--cases') args.cases = optionValue(argv, index++, arg);
    else if (arg === '--captures') args.captures = optionValue(argv, index++, arg);
    else if (arg === '--output') args.output = optionValue(argv, index++, arg);
    else if (arg === '--slug-prefix') args.slugPrefix = optionValue(argv, index++, arg);
    else if (arg === '--execution-class') args.executionClass = optionValue(argv, index++, arg);
    else if (arg === '--target-characters') args.targetCharacters = Number(optionValue(argv, index++, arg));
    else if (arg === '--hard-characters') args.hardCharacters = Number(optionValue(argv, index++, arg));
    else throw new Error(`Unknown argument: ${arg}`);
  }
  for (const [name, value] of Object.entries(args).filter(([name]) => !name.endsWith('Characters') && name !== 'executionClass')) {
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

export async function main(argv) {
  const args = parseArgs(argv);
  const pages = buildFailedPreviewRetryPlans(
    readJsonLines(args.cases),
    readJsonLines(args.captures),
    {
      slugPrefix: args.slugPrefix,
      executionClass: args.executionClass,
      targetCharacters: args.targetCharacters,
      hardCharacters: args.hardCharacters,
    },
  );
  fs.writeFileSync(args.output, `${pages.map((value) => JSON.stringify(value)).join('\n')}\n`, {flag: 'wx'});
  console.log(JSON.stringify({retry_pages: pages.length, retry_cases: pages.reduce((sum, page) => sum + page.cases.length, 0)}));
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
