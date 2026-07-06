// V3 golden-pair comparator for the local Wikidot rendering lab.
//
// Compares a locally rendered page against its frozen live Wikidot capture
// per specs/comparator-normalization.spec.md: every normalization channel is
// declared and logged; a difference that disappears only after normalization
// is a finding (`normalization_hides_visible_text_difference`), never a free
// pass. Local output is never its own oracle — the live side always comes
// from the frozen golden-pair evidence.

export const RENDER_COMPARE_SCHEMA = 'wikijump_local_lab.render_compare.v1';

const RAW_MARKER_PATTERNS = [
  /\[\[(?:module|include|\/module|\/include)\b/i,
  /content\s+not\s+rendered\s+yet/i,
  /corpus-shell-import/i,
];

const PLACEHOLDER_PATTERNS = [
  /\bcontent\s+not\s+rendered\s+yet\b/i,
  /\bnot\s+rendered\b/i,
  /\bplaceholder\s+(?:content|page|body|text|render(?:ed|ing)?|module|include)\b/i,
  /\b(?:content|page|body|text|render(?:ed|ing)?)\s+placeholder\b/i,
];

// Declared normalization channels (spec §3). Each is independently
// switchable; the applied set is recorded in the verdict. whitespace_collapse
// is implemented but ships OFF by default pending the spec's open question —
// enabling it requires an explicit flag so the decision is visible in logs.
export const DEFAULT_CHANNELS = {
  hostname_map: true,
  request_id: true,
  semantic_timestamp: true,
  cache_buster: true,
  env_id: true,
  whitespace_collapse: false,
};

const HOSTNAME_MAP = [
  [/scp-wiki\.wikidot\.com/g, '{{site-host}}'],
  [/scp-wiki\.wikijump\.localhost/g, '{{site-host}}'],
  [/scp-jp\.wikidot\.com/g, '{{site-host}}'],
  [/scp-jp\.localhost/g, '{{site-host}}'],
  [/www\.wikidot\.com/g, '{{platform-host}}'],
  [/www\.wikijump\.localhost/g, '{{platform-host}}'],
];

export function normalizeText(text, channels = DEFAULT_CHANNELS) {
  let result = text;
  const applied = [];
  if (channels.hostname_map) {
    applied.push('hostname_map');
    for (const [pattern, replacement] of HOSTNAME_MAP) result = result.replace(pattern, replacement);
  }
  if (channels.request_id) {
    applied.push('request_id');
    result = result
      .replace(/\b(?:request|trace|session)[-_]?id[=:]\s*[\w-]+/gi, '{{request-id}}')
      .replace(/\btoken[=:][\w.-]+/gi, '{{token}}');
  }
  if (channels.semantic_timestamp) {
    applied.push('semantic_timestamp');
    result = result
      .replace(/\b\d+\s+(?:second|minute|hour|day|week|month|year)s?\s+ago\b/gi, '{{relative-time}}')
      .replace(/\b\d{1,2}\s+\w{3}\s+\d{4}\b,?\s*\d{1,2}:\d{2}(?::\d{2})?/g, '{{timestamp}}');
  }
  if (channels.cache_buster) {
    applied.push('cache_buster');
    result = result.replace(/\?v=[\w.-]+/g, '');
  }
  if (channels.env_id) {
    applied.push('env_id');
    result = result.replace(/\b[0-9a-f]{12,64}\b/g, '{{hex-id}}');
  }
  if (channels.whitespace_collapse) {
    applied.push('whitespace_collapse');
    result = result.split(/\s+/).join(' ').trim();
  }
  return { text: result, applied };
}

export function hasRawMarker(text) {
  return RAW_MARKER_PATTERNS.some((pattern) => pattern.test(text));
}

export function isPlaceholder(text) {
  return PLACEHOLDER_PATTERNS.some((pattern) => pattern.test(text));
}

function firstDifference(a, b, radius = 60) {
  const length = Math.min(a.length, b.length);
  let i = 0;
  while (i < length && a[i] === b[i]) i += 1;
  return {
    offset: i,
    source: a.slice(Math.max(0, i - radius), i + radius).replace(/\s+/g, ' '),
    local: b.slice(Math.max(0, i - radius), i + radius).replace(/\s+/g, ' '),
  };
}

export function matchLedgerEntry(ledger, fixtureId, category) {
  return (
    ledger.find(
      (entry) =>
        entry.category === category &&
        (entry.fixture_ids?.includes(fixtureId) || entry.scope === '*'),
    ) ?? null
  );
}

/**
 * Compare one golden pair.
 *
 * @param {object} input
 * @param {string} input.fixtureId
 * @param {string} input.sourceVisibleText  frozen live-capture visible text
 * @param {string} input.localVisibleText   locally rendered visible text
 * @param {string} [input.sourceUrl]
 * @param {string} [input.localUrl]
 * @param {string} [input.sourceArtifact]
 * @param {string} [input.localArtifact]
 * @param {object} [input.channels]         normalization channel switches
 * @param {object[]} [input.ledger]         accepted-diff ledger entries
 */
export function comparePair(input) {
  const channels = { ...DEFAULT_CHANNELS, ...(input.channels ?? {}) };
  const ledger = input.ledger ?? [];
  const findings = [];

  if (input.sourceUrl && input.localUrl && input.sourceUrl === input.localUrl) {
    findings.push({ category: 'self_comparison', detail: `identical URLs: ${input.sourceUrl}` });
  }
  if (input.sourceArtifact && input.localArtifact && input.sourceArtifact === input.localArtifact) {
    findings.push({ category: 'self_comparison', detail: `identical artifacts: ${input.sourceArtifact}` });
  }
  if (!input.sourceVisibleText || !input.localVisibleText) {
    findings.push({ category: 'visible_text_pair_missing', detail: 'source or local visible text missing' });
  }

  const source = input.sourceVisibleText ?? '';
  const local = input.localVisibleText ?? '';

  if (source && hasRawMarker(source)) {
    findings.push({ category: 'raw_marker_visible', side: 'source', detail: 'raw FTML/placeholder marker in live text' });
  }
  if (local && (hasRawMarker(local) || isPlaceholder(local))) {
    findings.push({ category: 'raw_marker_visible', side: 'local', detail: 'raw FTML/placeholder marker in local text' });
  }

  let appliedChannels = [];
  if (source && local && source !== local) {
    const normalizedSource = normalizeText(source, channels);
    const normalizedLocal = normalizeText(local, channels);
    appliedChannels = normalizedSource.applied;
    if (normalizedSource.text === normalizedLocal.text) {
      // The difference is fully explained by declared channels. Whitespace
      // collapse hiding a visible-text difference is the spec's guard case.
      const withoutWhitespace = normalizeText(source, { ...channels, whitespace_collapse: false });
      const withoutWhitespaceLocal = normalizeText(local, { ...channels, whitespace_collapse: false });
      if (channels.whitespace_collapse && withoutWhitespace.text !== withoutWhitespaceLocal.text) {
        findings.push({
          category: 'normalization_hides_visible_text_difference',
          detail: 'difference disappears only under whitespace_collapse',
          ...firstDifference(withoutWhitespace.text, withoutWhitespaceLocal.text),
        });
      }
      // Differences explained by non-whitespace declared channels are
      // recorded (not silent) but do not fail the pair.
      findings.push({
        category: 'normalized_difference',
        detail: `difference explained by declared channels: ${appliedChannels.join(', ')}`,
        informational: true,
      });
    } else {
      findings.push({
        category: 'visible_text_difference',
        detail: 'visible text differs after declared normalization',
        ...firstDifference(normalizedSource.text, normalizedLocal.text),
      });
    }
  }

  const blocking = findings.filter((f) => !f.informational);
  let verdict = 'match';
  let ledgerRefs = [];
  if (blocking.length > 0) {
    const unexplained = [];
    for (const finding of blocking) {
      const entry = matchLedgerEntry(ledger, input.fixtureId, finding.category);
      if (entry) {
        finding.ledger = { category: entry.category, scope: entry.scope };
        ledgerRefs.push(entry);
      } else {
        unexplained.push(finding);
      }
    }
    verdict = unexplained.length === 0 ? 'accepted-diff' : 'regression';
  }

  return {
    fixture_id: input.fixtureId,
    verdict,
    findings,
    normalization_channels: Object.entries(channels)
      .filter(([, enabled]) => enabled)
      .map(([name]) => name),
    ledger_refs: ledgerRefs.map((entry) => `${entry.category}:${entry.scope}`),
  };
}

export function aggregateCompareVerdict({ runId, pairs }) {
  const counts = { match: 0, 'accepted-diff': 0, regression: 0 };
  for (const pair of pairs) counts[pair.verdict] += 1;
  return {
    verdict: {
      schema: RENDER_COMPARE_SCHEMA,
      run_id: runId,
      pairs,
      aggregate: {
        pairs_total: pairs.length,
        counts,
        regressions: pairs.filter((p) => p.verdict === 'regression').map((p) => p.fixture_id),
      },
    },
    exitCode: counts.regression > 0 ? 1 : 0,
  };
}
