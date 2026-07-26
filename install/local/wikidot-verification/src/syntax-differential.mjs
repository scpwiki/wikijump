import {createHash} from 'node:crypto';

import {parseFragment, serialize} from 'parse5';

import {compareSignatures, domSignature} from './oracle-fixtures.mjs';

export const SYNTAX_CASE_SCHEMA = 'wikijump_syntax_differential.syntax_case.v1';
export const WIKIDOT_REFERENCE_SCHEMA = 'wikijump_syntax_differential.wikidot_reference.v1';
export const FTML_RENDER_RESULT_SCHEMA = 'wikijump_syntax_differential.ftml_render_result.v1';
export const SYNTAX_COMPARISON_SCHEMA = 'wikijump_syntax_differential.syntax_comparison.v1';

export function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function isRfc3339(value) {
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|([+-])(\d{2}):(\d{2}))$/u.exec(value);
  if (!match) return false;
  const [, yearText, monthText, dayText, hourText, minuteText, secondText, , offsetHourText, offsetMinuteText] = match;
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return (
    year > 0 &&
    month >= 1 &&
    month <= 12 &&
    day >= 1 &&
    day <= days[month - 1] &&
    Number(hourText) <= 23 &&
    Number(minuteText) <= 59 &&
    Number(secondText) <= 59 &&
    (offsetHourText == null || Number(offsetHourText) <= 23) &&
    (offsetMinuteText == null || Number(offsetMinuteText) <= 59) &&
    Number.isFinite(Date.parse(value))
  );
}

export function visibleText(html) {
  const tokens = [];
  const blocks = new Set(['p', 'div', 'blockquote', 'li', 'dt', 'dd', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'tr', 'td', 'th', 'pre']);
  const hidden = new Set(['script', 'style', 'template']);
  const pushBreak = () => {
    if (tokens.at(-1)?.kind === 'normal') {
      tokens.at(-1).value = tokens.at(-1).value.trimEnd();
      if (!tokens.at(-1).value) tokens.pop();
    }
    if (tokens.at(-1)?.kind !== 'break') tokens.push({kind: 'break', value: '\n'});
  };
  const visit = (node, preformatted = false) => {
    if (node.nodeName === '#text') {
      let value = preformatted ? node.value : node.value.replace(/\s+/gu, ' ');
      if (!preformatted && tokens.at(-1)?.kind === 'break') value = value.trimStart();
      if (value) tokens.push({kind: preformatted ? 'pre' : 'normal', value});
      return;
    }
    if (!node.tagName || hidden.has(node.tagName)) return;
    if (node.tagName === 'br') {
      pushBreak();
      return;
    }
    const block = blocks.has(node.tagName);
    if (block) pushBreak();
    for (const child of node.childNodes ?? []) visit(child, preformatted || node.tagName === 'pre');
    if (block) pushBreak();
  };
  for (const node of parseFragment(html).childNodes) visit(node);
  while (tokens[0]?.kind === 'break') tokens.shift();
  while (tokens.at(-1)?.kind === 'break') tokens.pop();
  if (tokens[0]?.kind === 'normal') tokens[0].value = tokens[0].value.trimStart();
  if (tokens.at(-1)?.kind === 'normal') tokens.at(-1).value = tokens.at(-1).value.trimEnd();
  while (tokens[0]?.kind === 'normal' && !tokens[0].value) tokens.shift();
  while (tokens.at(-1)?.kind === 'normal' && !tokens.at(-1).value) tokens.pop();
  while (tokens[0]?.kind === 'break') tokens.shift();
  while (tokens.at(-1)?.kind === 'break') tokens.pop();
  return tokens.map((token) => token.value).join('');
}

function canonicalAttributeValue(node, name, value) {
  const classValue = node.attrs?.find((attr) => attr.name === 'class')?.value ?? '';
  if (
    node.tagName === 'a' &&
    classValue.split(/\s+/u).includes('bibcite') &&
    name === 'id'
  ) {
    return value.replace(/^(bibcite-\d+)-[0-9a-z]+$/u, '$1-volatile');
  }
  if (!['href', 'src'].includes(name)) return value;
  let url;
  try {
    url = new URL(value);
  } catch {
    return value;
  }
  if (
    name === 'src' &&
    url.hostname === 'www.wikidot.com' &&
    url.pathname === '/avatar.php'
  ) {
    url.searchParams.delete('timestamp');
    url.searchParams.delete('amp;timestamp');
    return url.href;
  }
  const match = /^(?<site>.+)\.(?:wdfiles\.com|wjfiles\.(?:com|localhost))$/u.exec(url.hostname);
  if (!match) return value;
  url.protocol = 'https:';
  url.hostname = `${match.groups.site}.files.invalid`;
  return url.href;
}

function canonicalWikidotEmail(node) {
  const classes = node.attrs?.find((attr) => attr.name === 'class')?.value.split(/\s+/u) ?? [];
  if (node.tagName !== 'span' || !classes.includes('wiki-email') || node.childNodes?.length !== 1) {
    return null;
  }
  const child = node.childNodes[0];
  if (child.nodeName !== '#text') return null;
  const separator = child.value.indexOf('#');
  if (separator <= 0 || child.value.slice(0, separator) !== child.value.slice(separator + 1)) {
    return null;
  }
  const address = [...child.value.slice(0, separator)].reverse().join('').replace('|', '@');
  return {
    type: 'element',
    name: 'span',
    namespace: node.namespaceURI,
    attrs: [
      {name: 'class', value: 'wiki-email', namespace: null, prefix: null},
      {name: 'style', value: 'visibility: visible;', namespace: null, prefix: null},
    ],
    children: [{
      type: 'element',
      name: 'a',
      namespace: node.namespaceURI,
      attrs: [{name: 'href', value: `mailto:${address}`, namespace: null, prefix: null}],
      children: [{type: 'text', value: address}],
    }],
  };
}

function canonicalNode(node, preformatted = false) {
  if (node.nodeName === '#text') {
    if (!preformatted && /^\s*$/u.test(node.value)) return null;
    return {type: 'text', value: node.value};
  }
  if (node.nodeName === '#comment') return {type: 'comment', value: node.data};
  if (!node.tagName) return null;
  const email = canonicalWikidotEmail(node);
  if (email) return email;
  return {
    type: 'element',
    name: node.tagName,
    namespace: node.namespaceURI,
    attrs: [...node.attrs]
      .map((attr) => ({
        name: attr.name,
        value: canonicalAttributeValue(node, attr.name, attr.value),
        namespace: attr.namespace ?? null,
        prefix: attr.prefix ?? null,
      }))
      .sort((left, right) =>
        `${left.namespace ?? ''}:${left.name}`.localeCompare(`${right.namespace ?? ''}:${right.name}`),
      ),
    children: (node.childNodes ?? [])
      .map((child) => canonicalNode(child, preformatted || ['pre', 'code', 'textarea'].includes(node.tagName)))
      .filter(Boolean),
  };
}

export function canonicalDom(html) {
  return parseFragment(html).childNodes.map((node) => canonicalNode(node)).filter(Boolean);
}

export function validateWikidotReference(reference) {
  if (reference?.schema !== WIKIDOT_REFERENCE_SCHEMA) {
    throw new Error('Wikidot reference schema is unsupported');
  }
  const syntaxCase = reference.syntax_case;
  if (syntaxCase?.schema !== SYNTAX_CASE_SCHEMA) {
    throw new Error('Wikidot reference syntax case schema is unsupported');
  }
  if (syntaxCase.wikidot_observation_tier !== 'page-preview') {
    throw new Error(`syntax case ${syntaxCase.case_id} is not preview-compatible`);
  }
  if (!['ftml', 'wikijump-runtime'].includes(syntaxCase.local_execution_tier)) {
    throw new Error(`syntax case ${syntaxCase.case_id} has an unsupported local execution tier`);
  }
  if (
    typeof syntaxCase.case_id !== 'string' ||
    !syntaxCase.case_id ||
    typeof syntaxCase.source !== 'string' ||
    typeof syntaxCase.title !== 'string' ||
    !syntaxCase.title
  ) {
    throw new Error('Wikidot reference syntax case is invalid');
  }
  const provenance = reference.provenance;
  if (
    provenance?.module !== 'edit/PagePreviewModule' ||
    provenance.authenticated !== false ||
    provenance.mutated !== false ||
    typeof provenance.site !== 'string' ||
    !provenance.site ||
    typeof provenance.site_domain !== 'string' ||
    !provenance.site_domain ||
    typeof provenance.wikidot_py_version !== 'string' ||
    !provenance.wikidot_py_version ||
    !/^[0-9a-f]{40}$/u.test(provenance.wikidot_py_commit) ||
    !/^[0-9a-f]{64}$/u.test(provenance.requirements_sha256)
  ) {
    throw new Error(`Wikidot reference provenance is invalid for ${syntaxCase.case_id}`);
  }
  if (
    typeof reference.captured_at !== 'string' ||
    !isRfc3339(reference.captured_at)
  ) {
    throw new Error(`Wikidot reference capture time is invalid for ${syntaxCase.case_id}`);
  }
  if (reference.source_sha256 !== sha256(syntaxCase.source)) {
    throw new Error(`Wikidot reference source hash is invalid for ${syntaxCase.case_id}`);
  }
  if (typeof reference.raw_html !== 'string' || reference.raw_html_sha256 !== sha256(reference.raw_html)) {
    throw new Error(`Wikidot reference HTML hash is invalid for ${syntaxCase.case_id}`);
  }
  return reference;
}

export function ftmlInputFromReference(reference) {
  validateWikidotReference(reference);
  if (reference.syntax_case.local_execution_tier !== 'ftml') {
    throw new Error(`syntax case ${reference.syntax_case.case_id} requires Wikijump runtime`);
  }
  return {
    schema: SYNTAX_CASE_SCHEMA,
    case_id: reference.syntax_case.case_id,
    source: reference.syntax_case.source,
    title: reference.syntax_case.title,
    page_context: {
      site: reference.provenance.site,
      page: '',
    },
  };
}

export function compareSyntaxReference(reference, renderResult) {
  validateWikidotReference(reference);
  const caseId = reference.syntax_case.case_id;
  if (
    renderResult?.schema !== FTML_RENDER_RESULT_SCHEMA ||
    renderResult.case_id !== caseId ||
    renderResult.status !== 'rendered' ||
    typeof renderResult.html !== 'string'
  ) {
    return {
      schema: SYNTAX_COMPARISON_SCHEMA,
      case_id: caseId,
      status: 'runner-error',
      detail: 'FTML render result is missing, mismatched, or unsuccessful',
    };
  }

  const wikidotHtml = reference.raw_html;
  const ftmlHtml = renderResult.html;
  const wikidotDom = canonicalDom(wikidotHtml);
  const ftmlDom = canonicalDom(ftmlHtml);
  const domTreeMatches = JSON.stringify(wikidotDom) === JSON.stringify(ftmlDom);
  const structureDiffs = compareSignatures(
    domSignature(serialize(parseFragment(wikidotHtml))),
    domSignature(serialize(parseFragment(ftmlHtml))),
  );
  const wikidotText = visibleText(wikidotHtml);
  const ftmlText = visibleText(ftmlHtml);
  const textMatches = wikidotText === ftmlText;
  const matches = domTreeMatches && structureDiffs.length === 0 && textMatches;
  return {
    schema: SYNTAX_COMPARISON_SCHEMA,
    case_id: caseId,
    status: matches ? 'match' : 'mismatch',
    checks: {
      dom_tree: {
        status: domTreeMatches ? 'match' : 'mismatch',
        ...(domTreeMatches ? {} : {wikidot: wikidotDom, ftml: ftmlDom}),
      },
      dom_signature: {
        status: structureDiffs.length === 0 ? 'match' : 'mismatch',
        diffs: structureDiffs,
      },
      visible_text: {
        status: textMatches ? 'match' : 'mismatch',
        wikidot: wikidotText,
        ftml: ftmlText,
      },
    },
    identities: {
      source_sha256: reference.source_sha256,
      wikidot_html_sha256: reference.raw_html_sha256,
      ftml_html_sha256: sha256(renderResult.html),
      ftml_engine: renderResult.engine ?? null,
    },
    ...(matches
      ? {}
      : {
          diagnostic: {
            wikidot_html: reference.raw_html,
            ftml_html: renderResult.html,
          },
        }),
  };
}

export function aggregateSyntaxComparisons(comparisons, execution = null) {
  const counts = {match: 0, mismatch: 0, 'runner-error': 0, 'not-applicable': 0};
  for (const comparison of comparisons) counts[comparison.status] += 1;
  return {
    schema: 'wikijump_syntax_differential.verdict.v1',
    execution,
    comparisons,
    summary: {
      total: comparisons.length,
      ...counts,
    },
  };
}
