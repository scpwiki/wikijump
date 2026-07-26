import {createHash} from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

export const LIVE_CASE_SCHEMA = 'wikijump_syntax_differential.live_case.v1';
export const PAGE_PLAN_SCHEMA = 'wikijump_syntax_differential.wikidot_page_plan.v1';
const RECORDED_BATCH_SOURCE_LIMIT = 7_500;

const RUNTIME_PATTERNS = [
  /\{\$[^}\r\n]+\}/u,
  /\[\[include(?:\s|\])/iu,
  /\[\[(?:iftags|ifcategory)(?:\s|\])/iu,
  /\[\[(?:file|user)(?:\s|\])/iu,
  /\[\[\[/u,
  /\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b/iu,
  /\[\[(?:tabs|tabview)(?:\s|\])/iu,
];

const ISOLATED_PATTERNS = [
  /\[!--/u,
  /--\]/u,
  /@@/u,
  /(?:<<|>>)/u,
  /\[\[(?:char|character)(?:\s|\])/iu,
  /\[\[(?:tt|mono|monospace)(?:\s|\])/iu,
  /\[\[\*?radio(?:\s|\])/iu,
  /\[\[(?:code|html|raw)(?:\s|\])/iu,
  /\[\[#/u,
  /\[\[(?:footnote|bibliography|toc|collapsible|equation)(?:\s|\])/iu,
];

export function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function walk(root) {
  const files = [];
  for (const entry of fs.readdirSync(root, {withFileTypes: true})) {
    const target = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...walk(target));
    else if (entry.isFile()) files.push(target);
  }
  return files;
}

function caseId(relativePath) {
  return relativePath
    .replaceAll(path.sep, '--')
    .replace(/(?:--)?input\.ftml$/u, '')
    .replace(/\.ftml$/u, '');
}

export function classifyFixture(relativePath, source, siblingNames = new Set()) {
  if (relativePath === 'test/include/elements/input.ftml') {
    return {
      execution_class: 'not-applicable',
      page_scope: 'batch-safe',
      reasons: ['ftml-only-include-elements'],
    };
  }
  const pathRequiresIsolation = /(?:^|\/)(?:fail|revert|malformed[^/]*|wikidot-invalid)(?:\/|$)/u.test(relativePath);
  const sourceRequiresIsolation =
    ISOLATED_PATTERNS.some((pattern) => pattern.test(source)) ||
    source.lastIndexOf('[[') > source.lastIndexOf(']]') ||
    /^[\t \r\n]/u.test(source) ||
    /(?:^|\n)(?:\+{1,6}\s|>{1,}\s?|\*+\s|#+\s|\|)/u.test(source) ||
    /(?:^|\n).+_\s*$/u.test(source) ||
    /\\\n?$/u.test(source);
  const isolationReasons = [];
  if (pathRequiresIsolation || siblingNames.has('errors.json')) isolationReasons.push('malformed-or-recovery');
  if (sourceRequiresIsolation) isolationReasons.push('position-or-boundary-sensitive');
  const pageScope = isolationReasons.length > 0 ? 'isolated' : 'batch-safe';
  const moduleNames = [...source.matchAll(/\[\[\s*module\s+([A-Za-z][A-Za-z0-9_-]*)/giu)]
    .map((match) => match[1].toLowerCase());
  if (
    RUNTIME_PATTERNS.some((pattern) => pattern.test(source)) ||
    moduleNames.some((name) => name !== 'css')
  ) {
    return {
      execution_class: 'wikijump-runtime',
      page_scope: pageScope,
      reasons: ['page-or-site-runtime', ...isolationReasons],
    };
  }
  if (moduleNames.length > 0) {
    return {
      execution_class: 'page-preview-isolated',
      page_scope: 'isolated',
      reasons: ['context-free-css-module'],
    };
  }
  if (pathRequiresIsolation || siblingNames.has('errors.json') || sourceRequiresIsolation) {
    return {
      execution_class: 'page-preview-isolated',
      page_scope: 'isolated',
      reasons: isolationReasons,
    };
  }
  return {
    execution_class: 'saved-page-batch',
    page_scope: 'batch-safe',
    reasons: ['conservative-static-pack-safe'],
  };
}

export function collectFtmlFixtureCases(ftmlRoot) {
  const roots = [path.join(ftmlRoot, 'test'), path.join(ftmlRoot, 'tests', 'fixtures')];
  const sources = [];
  for (const root of roots) {
    if (!fs.existsSync(root)) continue;
    for (const file of walk(root)) {
      const relativePath = path.relative(ftmlRoot, file);
      if (
        !relativePath.endsWith(`${path.sep}input.ftml`) &&
        !relativePath.startsWith(`tests${path.sep}fixtures${path.sep}`)
      ) {
        continue;
      }
      if (!relativePath.endsWith('.ftml')) continue;
      const source = fs.readFileSync(file, 'utf8');
      const siblingNames = new Set(fs.readdirSync(path.dirname(file)));
      const classification = classifyFixture(relativePath, source, siblingNames);
      sources.push({
        schema: LIVE_CASE_SCHEMA,
        case_id: caseId(relativePath),
        source,
        source_sha256: sha256(source),
        source_origin: {
          repository: 'Rokurolize/ftml',
          path: relativePath.split(path.sep).join('/'),
        },
        ...classification,
      });
    }
  }
  return sources.sort((left, right) => left.case_id.localeCompare(right.case_id));
}

export function collectFtmlRecordedCases(recordPaths) {
  const bySource = new Map();
  for (const recordPath of recordPaths) {
    const lines = fs.readFileSync(recordPath, 'utf8').split('\n').filter((line) => line.trim());
    for (const line of lines) {
      const record = JSON.parse(line);
      if (
        record?.schema !== 'ftml.test_source_record.v1' ||
        typeof record.source !== 'string' ||
        typeof record.stage !== 'string'
      ) {
        throw new Error(`invalid FTML source record in ${recordPath}`);
      }
      const sourceSha256 = sha256(record.source);
      const existing = bySource.get(sourceSha256);
      const origin = {
        record_path: recordPath,
        stage: record.stage,
        test_name: record.test_name ?? null,
        caller: record.caller,
      };
      if (existing) {
        existing.record_origins.push(origin);
        continue;
      }
      const sourceCharacters = [...record.source].length;
      const sourceBytes = Buffer.byteLength(record.source);
      let classification =
        sourceCharacters > 160_000 || sourceBytes > 500_000
          ? {
              execution_class: 'not-applicable',
              page_scope: 'isolated',
              reasons: ['exceeds-wikidot-single-page-limit'],
            }
          : classifyFixture(`record/${record.test_name ?? 'unnamed'}/input.ftml`, record.source);
      if (
        classification.execution_class === 'saved-page-batch' &&
        sourceCharacters > RECORDED_BATCH_SOURCE_LIMIT
      ) {
        classification = {
          execution_class: 'page-preview-isolated',
          page_scope: 'isolated',
          reasons: ['exceeds-observed-safe-batch-size'],
        };
      }
      bySource.set(sourceSha256, {
        schema: LIVE_CASE_SCHEMA,
        case_id: `record--${sourceSha256.slice(0, 24)}`,
        source: record.source,
        source_sha256: sourceSha256,
        source_origin: {repository: 'Rokurolize/ftml', kind: 'test-source-recorder'},
        record_origins: [origin],
        ...classification,
      });
    }
  }
  const cases = [...bySource.values()].sort((left, right) => left.case_id.localeCompare(right.case_id));
  if (new Set(cases.map((value) => value.case_id)).size !== cases.length) {
    throw new Error('recorded FTML case ID prefix collision');
  }
  return cases;
}

function marker(kind, nonce, ordinal) {
  return `WJDIFF_${kind}_${nonce}_${String(ordinal).padStart(6, '0')}`;
}

function embeddedCaseSource(value, nonce, ordinal) {
  const begin = marker('BEGIN', nonce, ordinal);
  const end = marker('END', nonce, ordinal);
  if (value.source.includes(nonce)) throw new Error(`case source collides with run nonce: ${value.case_id}`);
  return {
    begin,
    end,
    source: `${begin}\n\n${value.source}\n\n${end}`,
  };
}

function codePointLength(value) {
  return [...value].length;
}

export function buildSavedPagePlans(cases, options = {}) {
  const targetCharacters = options.targetCharacters ?? 8_000;
  const hardCharacters = options.hardCharacters ?? 9_000;
  const slugPrefix = options.slugPrefix ?? 'run-owned:ftml-diff-20260726';
  const executionClass = options.executionClass ?? 'saved-page-batch';
  if (targetCharacters <= 0 || hardCharacters < targetCharacters) throw new Error('page character limits are invalid');
  const batchCases = cases.filter((value) => value.execution_class === executionClass);
  const manifestSha256 = sha256(
    JSON.stringify(batchCases.map(({case_id, source_sha256}) => ({case_id, source_sha256}))),
  );
  const nonce = manifestSha256.slice(0, 32);
  const pages = [];
  let current = [];
  let currentLength = 0;
  for (const [index, value] of batchCases.entries()) {
    const embedded = value.page_scope === 'isolated'
      ? {begin: null, end: null, source: value.source}
      : embeddedCaseSource(value, nonce, index + 1);
    const embeddedLength = codePointLength(embedded.source);
    if (embeddedLength > hardCharacters) {
      throw new Error(`one case exceeds the hard page limit: ${value.case_id}`);
    }
    const plannedCase = {
      ...value,
      marker_begin: embedded.begin,
      marker_end: embedded.end,
      embedded_source: embedded.source,
    };
    if (value.page_scope === 'isolated') {
      if (current.length > 0) pages.push(current);
      pages.push([plannedCase]);
      current = [];
      currentLength = 0;
      continue;
    }
    const separatorLength = current.length === 0 ? 0 : 2;
    if (current.length > 0 && currentLength + separatorLength + embeddedLength > targetCharacters) {
      pages.push(current);
      current = [];
      currentLength = 0;
    }
    current.push(plannedCase);
    currentLength += (current.length === 1 ? 0 : separatorLength) + embeddedLength;
  }
  if (current.length > 0) pages.push(current);
  return pages.map((pageCases, pageIndex) => {
    const source = pageCases.map((value) => value.embedded_source).join('\n\n');
    if (codePointLength(source) > hardCharacters) throw new Error('generated page exceeds the hard character limit');
    return {
      schema: PAGE_PLAN_SCHEMA,
      page_index: pageIndex + 1,
      slug: `${slugPrefix}-${String(pageIndex + 1).padStart(3, '0')}`,
      title: `FTML differential ${String(pageIndex + 1).padStart(3, '0')}`,
      source,
      source_sha256: sha256(source),
      source_characters: [...source].length,
      source_bytes: Buffer.byteLength(source),
      manifest_sha256: manifestSha256,
      run_nonce: nonce,
      cases: pageCases.map(({case_id, source_sha256, page_scope, marker_begin, marker_end}) => ({
        case_id,
        source_sha256,
        page_scope,
        ...(page_scope === 'isolated' ? {} : {marker_begin, marker_end}),
      })),
    };
  });
}

export function isolateBatchInteractions(cases, verification) {
  const interacting = new Set(
    verification.comparisons
      .filter((value) => value.status === 'batch-context-interaction')
      .map((value) => value.case_id),
  );
  return cases.map((value) =>
    interacting.has(value.case_id)
      ? {
          ...value,
          execution_class: 'page-preview-isolated',
          page_scope: 'isolated',
          reasons: ['measured-batch-context-interaction'],
        }
      : value,
  );
}

function containsOrderedMarkers(html, page) {
  if (typeof html !== 'string') return false;
  if (page.cases.length === 1 && page.cases[0].page_scope === 'isolated') return true;
  let previous = -1;
  for (const value of page.cases) {
    const begin = html.indexOf(value.marker_begin, previous + 1);
    const end = html.indexOf(value.marker_end, begin + value.marker_begin.length);
    if (
      begin < 0 ||
      end < 0 ||
      html.indexOf(value.marker_begin, begin + value.marker_begin.length) >= 0 ||
      html.indexOf(value.marker_end, end + value.marker_end.length) >= 0
    ) {
      return false;
    }
    previous = end;
  }
  return true;
}

export function buildFailedPreviewRetryPlans(cases, captures, options = {}) {
  const byCaseId = new Map(cases.map((value) => [value.case_id, value]));
  const unresolved = new Set();
  for (const capture of captures) {
    const page = capture.page_plan;
    const resolved = containsOrderedMarkers(capture.page_content_html, page);
    for (const value of page.cases) {
      if (!byCaseId.has(value.case_id)) {
        throw new Error(`failed preview references an unknown case: ${value.case_id}`);
      }
      if (resolved) unresolved.delete(value.case_id);
      else unresolved.add(value.case_id);
    }
  }
  const retryCases = cases.filter((value) => unresolved.has(value.case_id));
  return buildSavedPagePlans(retryCases, {
    slugPrefix: options.slugPrefix,
    targetCharacters: options.targetCharacters ?? 8_000,
    hardCharacters: options.hardCharacters ?? 9_000,
    executionClass: options.executionClass ?? retryCases[0]?.execution_class,
  });
}

export function summarizeLiveCases(cases, pages) {
  const execution = {};
  for (const value of cases) execution[value.execution_class] = (execution[value.execution_class] ?? 0) + 1;
  return {
    total_cases: cases.length,
    total_source_characters: cases.reduce((sum, value) => sum + [...value.source].length, 0),
    total_source_bytes: cases.reduce((sum, value) => sum + Buffer.byteLength(value.source), 0),
    execution,
    saved_pages: pages.length,
    saved_page_characters: pages.map((value) => value.source_characters),
    saved_page_bytes: pages.map((value) => value.source_bytes),
  };
}
