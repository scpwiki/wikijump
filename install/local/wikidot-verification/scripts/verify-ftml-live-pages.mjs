#!/usr/bin/env node

import fs from 'node:fs';

import {parseFragment, serializeOuter} from 'parse5';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {canonicalDom} from '../src/syntax-differential.mjs';
import {renderCases} from './run-syntax-differential.mjs';

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {cases: null, pages: null, renderer: null, output: null, timeoutMs: 30_000};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--cases') args.cases = optionValue(argv, index++, arg);
    else if (arg === '--pages') args.pages = optionValue(argv, index++, arg);
    else if (arg === '--renderer') args.renderer = optionValue(argv, index++, arg);
    else if (arg === '--output') args.output = optionValue(argv, index++, arg);
    else if (arg === '--timeout-ms') args.timeoutMs = Number(optionValue(argv, index++, arg));
    else throw new Error(`Unknown argument: ${arg}`);
  }
  for (const name of ['cases', 'pages', 'renderer', 'output']) {
    if (!args[name]) throw new Error(`--${name} is required`);
  }
  if (!Number.isSafeInteger(args.timeoutMs) || args.timeoutMs <= 0) throw new Error('--timeout-ms must be a positive integer');
  return args;
}

function readJsonLines(path) {
  return fs.readFileSync(path, 'utf8').split('\n').filter((line) => line.trim()).map(JSON.parse);
}

function textContent(node) {
  if (node.nodeName === '#text') return node.value;
  return (node.childNodes ?? []).map(textContent).join('');
}

export function extractMarkedFragments(html, page) {
  const fragmentChildren = parseFragment(html).childNodes;
  const pageContent = fragmentChildren.find(
    (node) => node.tagName === 'div' && node.attrs?.some((attr) => attr.name === 'id' && attr.value === 'page-content'),
  );
  const children = pageContent?.childNodes ?? fragmentChildren;
  if (page.cases.length === 1 && page.cases[0].page_scope === 'isolated') {
    return new Map([[page.cases[0].case_id, children.map(serializeOuter).join('')]]);
  }
  const markerIndices = new Map();
  for (const [index, node] of children.entries()) {
    if (node.tagName !== 'p') continue;
    const text = textContent(node);
    if (page.cases.some((value) => value.marker_begin === text || value.marker_end === text)) {
      if (markerIndices.has(text)) throw new Error(`FTML marker integrity failed: duplicate ${text}`);
      markerIndices.set(text, index);
    }
  }
  const fragments = new Map();
  let previousEnd = -1;
  for (const value of page.cases) {
    const begin = markerIndices.get(value.marker_begin);
    const end = markerIndices.get(value.marker_end);
    if (begin == null || end == null || begin >= end || begin <= previousEnd) {
      return extractRawMarkedFragments(html, page);
    }
    fragments.set(value.case_id, children.slice(begin + 1, end).map(serializeOuter).join(''));
    previousEnd = end;
  }
  return fragments;
}

function uniqueMarkerIndex(html, marker, caseId) {
  const index = html.indexOf(marker);
  if (index < 0) {
    throw new Error(`FTML marker integrity failed for ${caseId}: missing ${marker}`);
  }
  if (html.indexOf(marker, index + marker.length) >= 0) {
    throw new Error(`FTML marker integrity failed for ${caseId}: duplicate ${marker}`);
  }
  return index;
}

function extractRawMarkedFragments(html, page) {
  const fragments = new Map();
  let previousEnd = -1;
  for (const value of page.cases) {
    const begin = uniqueMarkerIndex(html, value.marker_begin, value.case_id);
    const markerEnd = uniqueMarkerIndex(html, value.marker_end, value.case_id);
    if (begin <= previousEnd || begin >= markerEnd) {
      throw new Error(`FTML marker integrity failed for ${value.case_id}`);
    }
    let fragmentStart = begin + value.marker_begin.length;
    if (html.startsWith('</p>', fragmentStart)) fragmentStart += '</p>'.length;
    let fragmentEnd = markerEnd;
    if (html.slice(fragmentEnd - '<p>'.length, fragmentEnd) === '<p>') fragmentEnd -= '<p>'.length;
    fragments.set(value.case_id, html.slice(fragmentStart, fragmentEnd));
    previousEnd = markerEnd + value.marker_end.length;
  }
  return fragments;
}

export async function main(argv) {
  const args = parseArgs(argv);
  const cases = readJsonLines(args.cases);
  const pages = readJsonLines(args.pages);
  const byCaseId = new Map(cases.map((value) => [value.case_id, value]));
  const inputs = [];
  for (const page of pages) {
    inputs.push({
      schema: 'wikijump_syntax_differential.syntax_case.v1',
      case_id: `page:${page.page_index}`,
      source: page.source,
      title: page.title,
    });
    for (const value of page.cases) {
      const syntaxCase = byCaseId.get(value.case_id);
      if (!syntaxCase) throw new Error(`page references an unknown case: ${value.case_id}`);
      inputs.push({
        schema: 'wikijump_syntax_differential.syntax_case.v1',
        case_id: `solo:${value.case_id}`,
        source: syntaxCase.source,
        title: page.title,
      });
    }
  }
  const results = await renderCases(inputs, args.renderer, [], args.timeoutMs, (value) => value);
  const byResultId = new Map(results.map((value) => [value.case_id, value]));
  const comparisons = [];
  for (const page of pages) {
    const pageResult = byResultId.get(`page:${page.page_index}`);
    if (pageResult?.status !== 'rendered') throw new Error(`FTML page render failed: ${page.page_index}`);
    const fragments = extractMarkedFragments(pageResult.html, page);
    for (const value of page.cases) {
      const solo = byResultId.get(`solo:${value.case_id}`);
      if (solo?.status !== 'rendered') throw new Error(`FTML solo render failed: ${value.case_id}`);
      const fragment = fragments.get(value.case_id);
      const matches = JSON.stringify(canonicalDom(fragment)) === JSON.stringify(canonicalDom(solo.html));
      comparisons.push({
        case_id: value.case_id,
        status: matches ? 'match' : 'batch-context-interaction',
        solo_html: solo.html,
        batch_fragment_html: fragment,
      });
    }
  }
  const report = {
    schema: 'wikijump_syntax_differential.ftml_batch_verification.v1',
    summary: {
      total: comparisons.length,
      match: comparisons.filter((value) => value.status === 'match').length,
      batch_context_interaction: comparisons.filter((value) => value.status !== 'match').length,
    },
    comparisons,
  };
  fs.writeFileSync(args.output, `${JSON.stringify(report, null, 2)}\n`, {flag: 'wx'});
  console.log(JSON.stringify(report.summary));
  return report.summary.batch_context_interaction === 0 ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
