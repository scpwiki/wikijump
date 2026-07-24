#!/usr/bin/env node
// V4 oracle-fixture check (agent-runnable): render each Wikidot Oracle
// entry's wikitext snippet through the local runtime and compare the DOM
// signature against the frozen live-Wikidot capture in the entry.
//
// Requires the local lab runtime (Deepwell RPC) and an oracle JSONL file.
// Fixture pages are created/updated under a reserved slug prefix on the
// target site and reused across runs.
//
// Usage:
//   oracle-fixture-check.mjs --oracle <entries.jsonl> --output <verdict.json> \
//     [--api-url http://127.0.0.1:2747/jsonrpc] [--site-id 6000002] \
//     [--slug-prefix oracle-fixture-] [--run-id <id>] \
//     [--admin-email admin@wikijump] [--admin-pass ...]
//
// Exit codes: 0 all pass, 1 failures, 2 structural (skipped entries/crash).

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';

import { aggregateOracleVerdict, compareOracleEntry } from '../src/oracle-fixtures.mjs';

const DEFAULT_API_URL = 'http://127.0.0.1:2747/jsonrpc';

export function usage() {
  return 'Usage: oracle-fixture-check.mjs --oracle <entries.jsonl> --output <verdict.json> ' +
    '[--api-url url] [--site-id id] [--slug-prefix p] [--run-id id]';
}

export function parseArgs(argv) {
  const args = {
    oracle: null,
    output: null,
    apiUrl: DEFAULT_API_URL,
    siteId: 6000002,
    slugPrefix: 'oracle-fixture-',
    runId: `v4-${new Date().toISOString().replace(/[:.]/g, '-')}`,
    adminEmail: process.env.WIKIDOT_VERIFY_ADMIN_EMAIL ?? 'admin@wikijump',
    adminPass: process.env.WIKIDOT_VERIFY_ADMIN_PASS ?? 'wikijumpadmin1',
    userId: -1,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => argv[++i];
    if (arg === '--oracle') args.oracle = next();
    else if (arg === '--output') args.output = next();
    else if (arg === '--api-url') args.apiUrl = next();
    else if (arg === '--site-id') args.siteId = Number(next());
    else if (arg === '--slug-prefix') args.slugPrefix = next();
    else if (arg === '--run-id') args.runId = next();
    else if (arg === '--admin-email') args.adminEmail = next();
    else if (arg === '--admin-pass') args.adminPass = next();
    else if (arg === '--user-id') args.userId = Number(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.oracle) throw new Error('--oracle is required');
  if (!args.output) throw new Error('--output is required');
  return args;
}

let rpcId = 0;
async function rpc(args, method, params, sessionToken = null) {
  const headers = { 'Content-Type': 'application/json' };
  if (sessionToken) headers['X-Deepwell-Session-Token'] = sessionToken;
  const response = await fetch(args.apiUrl, {
    method: 'POST',
    headers,
    body: JSON.stringify({ jsonrpc: '2.0', id: ++rpcId, method, params }),
  });
  const body = await response.json();
  if (body.error) throw new Error(`${method} failed: ${JSON.stringify(body.error)}`);
  return body.result;
}

function fixtureSlug(args, entry) {
  // sentinel id when present, else the oracle entry id (already unique).
  const sentinel = entry.live_pilot_provenance?.sentinel_id;
  const base = (sentinel ?? entry.oracle_entry_id).toLowerCase().replace(/[^a-z0-9-]+/g, '-');
  return `${args.slugPrefix}${base}`;
}

async function renderEntry(args, sessionToken, entry) {
  const slug = fixtureSlug(args, entry);
  const wikitext = entry.source;
  let page = null;
  try {
    page = await rpc(
      args,
      'page_get',
      { site_id: args.siteId, page: slug, details: { wikitext: true, compiled: true } },
      sessionToken,
    );
  } catch {
    page = null;
  }
  const common = {
    site_id: args.siteId,
    wikitext,
    title: entry.oracle_entry_id,
    user_id: args.userId,
    ip_address: '127.0.0.1',
  };
  if (!page) {
    await rpc(
      args,
      'page_create',
      { ...common, alt_title: null, slug, layout: 'wikidot', revision_comments: 'v4 oracle fixture', tags: ['oracle-fixture'] },
      sessionToken,
    );
  } else if (page.wikitext !== wikitext) {
    await rpc(
      args,
      'page_edit',
      { ...common, page: page.page_id, last_revision_id: page.revision_id, revision_comments: 'v4 oracle fixture update', tags: ['oracle-fixture'] },
      sessionToken,
    );
  }
  const rendered = await rpc(
    args,
    'page_get',
    { site_id: args.siteId, page: slug, details: { compiled: true } },
    sessionToken,
  );
  return rendered.compiled_body_html ?? '';
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const entries = fs
    .readFileSync(args.oracle, 'utf8')
    .trim()
    .split('\n')
    .filter(Boolean)
    .map((line) => JSON.parse(line));

  const login = await rpc(args, 'login', {
    name_or_email: args.adminEmail,
    password: args.adminPass,
    ip_address: '127.0.0.1',
    user_agent: 'oracle-fixture-check/0.1',
  });

  const results = [];
  for (const entry of entries) {
    try {
      const html = await renderEntry(args, login.session_token, entry);
      results.push(compareOracleEntry(entry, html));
    } catch (error) {
      results.push({
        oracle_entry_id: entry.oracle_entry_id,
        status: 'skipped',
        reason: String(error).slice(0, 300),
      });
    }
  }

  const { verdict, exitCode } = aggregateOracleVerdict({ runId: args.runId, results });
  fs.writeFileSync(args.output, JSON.stringify(verdict, null, 1));
  console.log(JSON.stringify({ run_id: args.runId, ...verdict.aggregate }, null, 2));
  return exitCode;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
