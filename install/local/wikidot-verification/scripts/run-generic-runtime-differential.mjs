#!/usr/bin/env node

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  DeepwellRpcAdapter,
  runGenericRuntimeDifferential,
} from '../src/generic-runtime-differential.mjs';
import {sha256} from '../src/syntax-differential.mjs';

function valueAfter(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${option} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    cases: null,
    captures: [],
    externalReferences: [],
    stateFixtures: [],
    runtimeIdentity: null,
    rpcUrl: null,
    textBlockUrl: null,
    site: 'sandbox-for-codex',
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === '--cases') args.cases = valueAfter(argv, index++, option);
    else if (option === '--captures') args.captures.push(valueAfter(argv, index++, option));
    else if (option === '--state-fixture') {
      args.stateFixtures.push(valueAfter(argv, index++, option));
    }
    else if (option === '--external-reference') {
      args.externalReferences.push(valueAfter(argv, index++, option));
    } else if (option === '--runtime-identity') {
      args.runtimeIdentity = valueAfter(argv, index++, option);
    } else if (option === '--rpc-url') args.rpcUrl = valueAfter(argv, index++, option);
    else if (option === '--text-block-url') {
      args.textBlockUrl = valueAfter(argv, index++, option);
    }
    else if (option === '--site') args.site = valueAfter(argv, index++, option);
    else if (option === '--output') args.output = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  for (const field of ['cases', 'runtimeIdentity', 'rpcUrl', 'textBlockUrl', 'output']) {
    if (!args[field]) throw new Error(`--${field.replace(/[A-Z]/gu, (value) => `-${value.toLowerCase()}`)} is required`);
  }
  if (args.captures.length === 0) throw new Error('--captures is required');
  if (args.site !== 'sandbox-for-codex') throw new Error('--site must be sandbox-for-codex');
  return args;
}

function readJsonLines(path) {
  return fs.readFileSync(path, 'utf8').split('\n').filter((line) => line.trim()).map(JSON.parse);
}

function fileIdentity(path) {
  const bytes = fs.readFileSync(path);
  return {path, sha256: sha256(bytes)};
}

export async function main(argv) {
  const args = parseArgs(argv);
  const administratorEmail = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL;
  const administratorPassword = process.env.WIKIDOT_VERIFY_ADMIN_PASS;
  if (!administratorEmail || !administratorPassword) {
    throw new Error('WIKIDOT_VERIFY_ADMIN_EMAIL and WIKIDOT_VERIFY_ADMIN_PASS are required');
  }
  const adapter = new DeepwellRpcAdapter({
    rpcUrl: args.rpcUrl,
    textBlockBaseUrl: args.textBlockUrl,
    siteSlug: args.site,
    administratorEmail,
    administratorPassword,
  });
  try {
    const report = await runGenericRuntimeDifferential({
      cases: readJsonLines(args.cases).filter(
        (value) => value.execution_class === 'wikijump-runtime',
      ),
      captureFiles: args.captures.map((path) => ({path, captures: readJsonLines(path)})),
      externalReferences: args.externalReferences.flatMap(readJsonLines),
      runtimeIdentity: JSON.parse(fs.readFileSync(args.runtimeIdentity, 'utf8')),
      adapter,
      stateFixtures: args.stateFixtures.map((path) => ({
        path,
        sha256: fileIdentity(path).sha256,
        fixture: JSON.parse(fs.readFileSync(path, 'utf8')),
      })),
      disposableRunId: process.env.WIKIDOT_VERIFY_DISPOSABLE_RUN_ID ?? null,
      inputIdentities: {
        cases: fileIdentity(args.cases),
        captures: args.captures.map(fileIdentity),
        external_references: args.externalReferences.map(fileIdentity),
        state_fixtures: args.stateFixtures.map(fileIdentity),
        runtime_identity: fileIdentity(args.runtimeIdentity),
      },
    });
    fs.writeFileSync(args.output, `${JSON.stringify(report, null, 2)}\n`, {flag: 'wx'});
    console.log(JSON.stringify(report.summary));
    return report.status === 'pass' ? 0 : 1;
  } finally {
    await adapter.close();
  }
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error.stack ?? error);
    return 2;
  },
});
