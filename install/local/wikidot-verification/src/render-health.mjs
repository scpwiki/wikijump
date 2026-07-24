// V2 render-health classification for the local Wikidot rendering lab.
//
// Implements the category taxonomy, severity levels, dispositions, verdict
// JSON shape, and exit-code discipline from
// local-wikidot-lab-redesign-20260705/specs/render-health-taxonomy.spec.md.
//
// V2 is an intrinsic (robustness) check: it asks whether a locally rendered
// page looks structurally sane. It never makes fidelity claims — those belong
// to the V3/V4/V5 comparators.

export const RENDER_HEALTH_SCHEMA = 'wikijump_local_lab.render_health.v1';

// Raw wikitext markers that should not appear as visible text in a healthy
// render. Escaped display copies (@@[[include ...]]@@ in source) legitimately
// render these literally; callers pass the page source so classification can
// discount markers the author escaped on purpose.
const LEAK_MARKERS = [
  // Empty `[[include]]` text occurs as prose in real theme articles. Only an include with an argument can represent an unresolved directive.
  { pattern: /\[\[include\s+[^\]\s]/gi, category: 'unresolved-include' },
  { pattern: /\[\[module\s+\w+/gi, category: 'leaked-marker', marker: 'module' },
  { pattern: /\[\[\/module\]\]/gi, category: 'leaked-marker', marker: 'module-close' },
  { pattern: /\[\[image\b/gi, category: 'leaked-marker', marker: 'image' },
  { pattern: /\[\[collapsible\b/gi, category: 'leaked-marker', marker: 'collapsible' },
  { pattern: /\[\[tabview\b/gi, category: 'leaked-marker', marker: 'tabview' },
  { pattern: /\[\[div\b/gi, category: 'leaked-marker', marker: 'div' },
  { pattern: /\[\[span\b/gi, category: 'leaked-marker', marker: 'span' },
  { pattern: /\[\[iftags\b/gi, category: 'leaked-marker', marker: 'iftags' },
  { pattern: /%%content%%/gi, category: 'leaked-marker', marker: 'content-variable' },
];

const CATEGORY_SEVERITY = {
  'leaked-marker': 'S3',
  'unresolved-include': 'S3',
  'module-placeholder-expected': 'S1',
  'missing-local-asset': 'S3',
  'failed-request': 'S3',
  'parser-error': 'S3',
  'console-error': 'S3',
  'local-runtime-error': 'S4',
  'route-missing': 'S4',
  'empty-render': 'S4',
  'unsupported-known': 'S2',
  'taxonomy-unknown': 'S4',
};

const SEVERITY_ORDER = ['S0', 'S1', 'S2', 'S3', 'S4'];

const SEVERITY_STATUS = {
  S0: 'pass',
  S1: 'pass-with-warnings',
  S2: 'unsupported-known',
  S3: 'failed',
  S4: 'failed',
};

function maxSeverity(a, b) {
  return SEVERITY_ORDER.indexOf(a) >= SEVERITY_ORDER.indexOf(b) ? a : b;
}

const RENDERED_LITERAL_PATTERNS = [
  /<span\b(?=[^>]*\bclass\s*=\s*(?:"[^"]*\bwj-raw\b[^"]*"|'[^']*\bwj-raw\b[^']*'))[^>]*>[\s\S]*?<\/span\s*>/gi,
  /<span\b(?=[^>]*\bstyle\s*=\s*(?:"[^"]*\bwhite-space\s*:\s*pre-wrap\b[^"]*"|'[^']*\bwhite-space\s*:\s*pre-wrap\b[^']*'))[^>]*>[\s\S]*?<\/span\s*>/gi,
];

// Strip regions that legitimately contain raw markup: scripts, styles, rendered code blocks, and FTML's identifiable literal spans.
const NON_CONTENT_PATTERNS = [
  /<script\b[\s\S]*?<\/script\b[^>]*>/gi,
  /<style\b[\s\S]*?<\/style\b[^>]*>/gi,
  /<pre\b[\s\S]*?<\/pre\b[^>]*>/gi,
  /<code\b[\s\S]*?<\/code\b[^>]*>/gi,
  ...RENDERED_LITERAL_PATTERNS,
];

export function stripNonContent(html) {
  // Iterate to a fixed point so removal cannot splice new open/close pairs
  // back together (single-pass replace can leave residual markers).
  let stripped = html;
  for (;;) {
    let next = stripped;
    for (const pattern of NON_CONTENT_PATTERNS) {
      next = next.replace(pattern, '');
    }
    if (next === stripped) return stripped;
    stripped = next;
  }
}

// Count occurrences of a marker in the source that the author escaped for
// display (@@...@@ raw spans or code blocks). Those occurrences are expected
// to appear literally in the rendered page and must not count as leaks.
export function countEscapedOccurrences(source, pattern, {includeCodeBlocks = true} = {}) {
  if (!source) return 0;
  let count = 0;
  const escapedRegions = [
    ...source.matchAll(/@@([\s\S]*?)@@/g),
    ...(includeCodeBlocks ? source.matchAll(/\[\[code[^\]]*\]\]([\s\S]*?)\[\[\/code\]\]/gi) : []),
  ];
  for (const region of escapedRegions) {
    const matches = region[1].match(new RegExp(pattern.source, pattern.flags));
    if (matches) count += matches.length;
  }
  return count;
}

function excerptAround(html, index, radius = 80) {
  const start = Math.max(0, index - radius);
  const end = Math.min(html.length, index + radius);
  return html.slice(start, end).replace(/\s+/g, ' ').trim();
}

export function findRawSyntaxLeaks({ html = '', source = '' } = {}) {
  const content = stripNonContent(html);
  const findings = [];
  const renderedLiteralsIdentified = RENDERED_LITERAL_PATTERNS.some((pattern) => new RegExp(pattern.source, pattern.flags).test(html));

  for (const { pattern, category, marker } of LEAK_MARKERS) {
    const matches = [...content.matchAll(new RegExp(pattern.source, pattern.flags))];
    if (matches.length === 0) continue;
    const escaped = renderedLiteralsIdentified ? 0 : countEscapedOccurrences(source, pattern, {includeCodeBlocks: false});
    const leaked = Math.max(0, matches.length - escaped);
    if (leaked === 0) continue;
    const firstLeak = matches[Math.min(escaped, matches.length - 1)];
    findings.push({
      category,
      marker,
      count: leaked,
      pattern: pattern.source,
      text: firstLeak[0],
      context: excerptAround(content, firstLeak.index),
    });
  }

  return findings;
}

/**
 * Classify one rendered page.
 *
 * @param {object} input
 * @param {string} input.fixtureId    stable id (e.g. "EN:scp-173")
 * @param {number} input.httpStatus   HTTP status of the page fetch (0 = network error)
 * @param {string} [input.html]       rendered HTML (required when httpStatus is 200)
 * @param {string} [input.source]     original wikitext, used to discount escaped markers
 * @param {string[]} [input.parserErrors]  parser/render errors reported for the page
 * @param {string[]} [input.consoleErrors] browser console errors (browser-capture mode)
 * @param {string[]} [input.failedRequests] failed subresource URLs (browser-capture mode)
 * @param {object} [input.dispositions]    category -> 'defect'|'known-unsupported'|'accepted'
 */
export function classifyRenderedPage(input) {
  const findings = [];

  if (input.httpStatus === 404) {
    findings.push({ category: 'route-missing', detail: 'page route returned 404' });
  } else if (input.httpStatus === 0 || input.httpStatus >= 500) {
    findings.push({
      category: 'local-runtime-error',
      detail: `page fetch returned ${input.httpStatus === 0 ? 'network error' : input.httpStatus}`,
    });
  } else if (input.httpStatus === 200) {
    const html = input.html ?? '';
    const content = stripNonContent(html);

    const bodyText = content
      .replace(/<[^>]+>/g, ' ')
      .replace(/\s+/g, ' ')
      .trim();
    const sourceIsEmpty = (input.source ?? '').trim() === '';
    if (bodyText === '' && !sourceIsEmpty) {
      findings.push({ category: 'empty-render', detail: 'rendered body empty for non-empty source' });
    }

    findings.push(...findRawSyntaxLeaks({ html, source: input.source }).map((finding) => ({
      category: finding.category,
      marker: finding.marker,
      count: finding.count,
      detail: finding.context,
    })));

    for (const url of input.failedRequests ?? []) {
      const category = /\/local--files\//.test(url) ? 'missing-local-asset' : 'failed-request';
      findings.push({ category, detail: url });
    }
    for (const err of input.consoleErrors ?? []) {
      findings.push({ category: 'console-error', detail: String(err).slice(0, 300) });
    }
  } else {
    // Redirects and other unexpected statuses have no taxonomy entry: block.
    findings.push({
      category: 'taxonomy-unknown',
      detail: `unhandled HTTP status ${input.httpStatus} (follow redirects or extend taxonomy)`,
    });
  }

  for (const err of input.parserErrors ?? []) {
    findings.push({ category: 'parser-error', detail: String(err).slice(0, 300) });
  }

  const dispositions = input.dispositions ?? {};
  let severity = 'S0';
  let needsReview = false;
  for (const finding of findings) {
    const disposition = dispositions[finding.category] ?? 'defect';
    finding.disposition = finding.category === 'taxonomy-unknown' ? 'needs-review' : disposition;
    if (finding.category === 'taxonomy-unknown') needsReview = true;
    if (finding.disposition === 'defect' || finding.disposition === 'needs-review') {
      severity = maxSeverity(severity, CATEGORY_SEVERITY[finding.category] ?? 'S4');
    } else if (finding.disposition === 'known-unsupported') {
      severity = maxSeverity(severity, 'S2');
    } else if (finding.disposition === 'accepted') {
      severity = maxSeverity(severity, 'S1');
    }
  }

  return {
    fixture_id: input.fixtureId,
    status: SEVERITY_STATUS[severity],
    severity,
    categories: [...new Set(findings.map((f) => f.category))],
    findings,
    needs_review: needsReview,
    excerpt: findings[0]?.detail ?? '',
  };
}

export function aggregateVerdict({ runId, family, pages, threshold = null }) {
  const categoryCounts = {};
  let healthy = 0;
  let needsReview = false;
  for (const page of pages) {
    if (page.status === 'pass' || page.status === 'pass-with-warnings') healthy += 1;
    if (page.needs_review) needsReview = true;
    for (const category of page.categories) {
      categoryCounts[category] = (categoryCounts[category] ?? 0) + 1;
    }
  }
  const healthRate = pages.length === 0 ? 0 : healthy / pages.length;
  const verdict = {
    schema: RENDER_HEALTH_SCHEMA,
    run_id: runId,
    family,
    pages: pages.map((page) => {
      const summary = {...page};
      delete summary.findings;
      delete summary.needs_review;
      return summary;
    }),
    aggregate: {
      family,
      pages_total: pages.length,
      pages_healthy: healthy,
      health_rate: Number(healthRate.toFixed(4)),
      category_counts: categoryCounts,
    },
  };
  let exitCode = 0;
  if (needsReview) exitCode = 2;
  else if (threshold !== null && healthRate < threshold) exitCode = 1;
  return { verdict, exitCode, findingsByPage: pages };
}

const HTML_ESCAPES = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };

function escapeHtml(text) {
  return String(text).replace(/[&<>"]/g, (c) => HTML_ESCAPES[c]);
}

export function renderDashboardHtml({ verdict, importSummary = null, previous = null, worstLimit = 20 }) {
  const { aggregate } = verdict;
  const worst = verdict.pages
    .filter((p) => p.status === 'failed' || p.status === 'unsupported-known')
    .slice(0, worstLimit);
  const trend =
    previous?.aggregate != null
      ? (aggregate.health_rate - previous.aggregate.health_rate).toFixed(4)
      : null;
  const categoryRows = Object.entries(aggregate.category_counts)
    .sort((a, b) => b[1] - a[1])
    .map(([category, count]) => `<tr><td>${escapeHtml(category)}</td><td>${count}</td></tr>`)
    .join('\n');
  const worstRows = worst
    .map(
      (p) =>
        `<tr><td>${escapeHtml(p.fixture_id)}</td><td>${escapeHtml(p.severity)}</td>` +
        `<td>${escapeHtml(p.categories.join(', '))}</td><td><code>${escapeHtml(p.excerpt)}</code></td></tr>`,
    )
    .join('\n');
  const importBlock = importSummary
    ? `<p>Import: ${importSummary.imported}/${importSummary.total} ` +
      `(${((importSummary.imported / Math.max(1, importSummary.total)) * 100).toFixed(1)}%)</p>`
    : '';
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Lab health — ${escapeHtml(verdict.family)} ${escapeHtml(verdict.run_id)}</title>
<style>body{font-family:system-ui,sans-serif;margin:2rem;max-width:70rem}table{border-collapse:collapse}td,th{border:1px solid #ccc;padding:.3rem .6rem;text-align:left}code{white-space:pre-wrap}</style>
</head><body>
<h1>Render health — family ${escapeHtml(verdict.family)}</h1>
<p>Run <code>${escapeHtml(verdict.run_id)}</code> · schema <code>${escapeHtml(verdict.schema)}</code></p>
${importBlock}
<p>Health: <strong>${aggregate.pages_healthy}/${aggregate.pages_total}</strong> (${(aggregate.health_rate * 100).toFixed(1)}%)${trend !== null ? ` · trend vs previous: ${trend}` : ''}</p>
<h2>Category counts</h2>
<table><tr><th>Category</th><th>Pages</th></tr>
${categoryRows}
</table>
<h2>Worst pages (${worst.length})</h2>
<table><tr><th>Fixture</th><th>Severity</th><th>Categories</th><th>Excerpt</th></tr>
${worstRows}
</table>
<p><a href="verdict.json">Full verdict JSON</a></p>
</body></html>
`;
}
