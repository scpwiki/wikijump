#!/usr/bin/env node

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {canonicalDom, sha256, visibleText} from '../src/syntax-differential.mjs';
import {renderCases} from './run-syntax-differential.mjs';
import {extractMarkedFragments} from './verify-ftml-live-pages.mjs';

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {cases: null, captures: [], renderer: null, output: null, timeoutMs: 30_000};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--cases') args.cases = optionValue(argv, index++, arg);
    else if (arg === '--captures') args.captures.push(optionValue(argv, index++, arg));
    else if (arg === '--renderer') args.renderer = optionValue(argv, index++, arg);
    else if (arg === '--output') args.output = optionValue(argv, index++, arg);
    else if (arg === '--timeout-ms') args.timeoutMs = Number(optionValue(argv, index++, arg));
    else throw new Error(`Unknown argument: ${arg}`);
  }
  for (const name of ['cases', 'renderer', 'output']) {
    if (!args[name]) throw new Error(`--${name} is required`);
  }
  if (args.captures.length === 0) throw new Error('--captures is required');
  if (!Number.isSafeInteger(args.timeoutMs) || args.timeoutMs <= 0) throw new Error('--timeout-ms must be a positive integer');
  return args;
}

function readJsonLines(path) {
  return fs.readFileSync(path, 'utf8').split('\n').filter((line) => line.trim()).map(JSON.parse);
}

export function compareFragment(caseId, wikidotHtml, ftmlHtml) {
  const wikidotDom = canonicalDom(wikidotHtml);
  const ftmlDom = canonicalDom(ftmlHtml);
  const wikidotText = visibleText(wikidotHtml);
  const ftmlText = visibleText(ftmlHtml);
  const domMatches = JSON.stringify(wikidotDom) === JSON.stringify(ftmlDom);
  const textMatches = wikidotText === ftmlText;
  return {
    case_id: caseId,
    status: domMatches && textMatches ? 'match' : 'mismatch',
    checks: {
      dom_tree: {status: domMatches ? 'match' : 'mismatch'},
      visible_text: {
        status: textMatches ? 'match' : 'mismatch',
        wikidot: wikidotText,
        ftml: ftmlText,
      },
    },
    identities: {
      wikidot_html_sha256: sha256(wikidotHtml),
      ftml_html_sha256: sha256(ftmlHtml),
    },
    ...(domMatches && textMatches ? {} : {diagnostic: {wikidot_html: wikidotHtml, ftml_html: ftmlHtml}}),
  };
}

export async function main(argv) {
  const args = parseArgs(argv);
  const cases = readJsonLines(args.cases);
  const captures = args.captures.flatMap(readJsonLines);
  const byCaseId = new Map(cases.map((value) => [value.case_id, value]));
  const inputs = [];
  const liveFragments = new Map();
  const provenance = new Map();
  const unresolved = new Set();
  for (const capture of captures) {
    const page = capture.page_plan;
    let fragments;
    try {
      fragments = extractMarkedFragments(capture.page_content_html, page);
    } catch {
      for (const value of page.cases) unresolved.add(value.case_id);
      continue;
    }
    for (const value of page.cases) {
      const syntaxCase = byCaseId.get(value.case_id);
      if (!syntaxCase) throw new Error(`capture references an unknown case: ${value.case_id}`);
      const fragment = fragments.get(value.case_id);
      if (fragment == null) {
        unresolved.add(value.case_id);
        continue;
      }
      liveFragments.set(value.case_id, fragment);
      unresolved.delete(value.case_id);
      provenance.set(value.case_id, {
        captured_at: capture.captured_at,
        site: capture.site,
        domain: capture.domain,
        page_identity: capture.page_identity,
        page_source_sha256: page.source_sha256,
      });
      inputs.push({
        schema: 'wikijump_syntax_differential.syntax_case.v1',
        case_id: value.case_id,
        source: syntaxCase.source,
        title: page.title,
      });
    }
  }
  if (unresolved.size > 0) {
    throw new Error(`capture retries did not resolve ${unresolved.size} cases`);
  }
  const rendered = await renderCases(inputs, args.renderer, [], args.timeoutMs, (value) => value);
  const byRenderedId = new Map(rendered.map((value) => [value.case_id, value]));
  const comparisons = [...liveFragments].map(([caseId, wikidotHtml]) => {
    const ftml = byRenderedId.get(caseId);
    if (ftml?.status !== 'rendered') throw new Error(`FTML result is unsuccessful: ${caseId}`);
    return {...compareFragment(caseId, wikidotHtml, ftml.html), provenance: provenance.get(caseId), ftml_engine: ftml.engine};
  });
  const report = {
    schema: 'wikijump_syntax_differential.wikidot_saved_page_verdict.v1',
    summary: {
      total: comparisons.length,
      match: comparisons.filter((value) => value.status === 'match').length,
      mismatch: comparisons.filter((value) => value.status === 'mismatch').length,
    },
    comparisons,
  };
  fs.writeFileSync(args.output, `${JSON.stringify(report, null, 2)}\n`, {flag: 'wx'});
  console.log(JSON.stringify(report.summary));
  return report.summary.mismatch === 0 ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
