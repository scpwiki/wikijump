import {canonicalDom, sha256, validateWikidotReference, visibleText} from './syntax-differential.mjs';
import {
  bindLocalHtmlBlockPayloads,
  countLocalHtmlBlockHandles,
  projectRuntimeHtmlBlocks,
  sha1,
} from './runtime-html-blocks.mjs';
import {validateRuntimeIdentity} from './saved-page-runtime-differential.mjs';
import {extractMarkedFragments} from '../scripts/verify-ftml-live-pages.mjs';

export const REPORT_SCHEMA = 'wikijump_syntax_differential.generic_runtime_verdict.v1';
export const CAPTURE_SCHEMA = 'wikijump_syntax_differential.wikidot_saved_page_capture.v1';
export const LIVE_CASE_SCHEMA = 'wikijump_syntax_differential.live_case.v1';

export class RuntimeCleanupError extends Error {}

function assertSha(value, name) {
  if (typeof value !== 'string' || !/^[0-9a-f]{64}$/u.test(value)) {
    throw new Error(`${name} is invalid`);
  }
}

function validateCase(value) {
  if (
    value?.schema !== LIVE_CASE_SCHEMA ||
    typeof value.case_id !== 'string' ||
    typeof value.source !== 'string' ||
    value.execution_class !== 'wikijump-runtime' ||
    (value.page_scope != null && !['batch-safe', 'isolated'].includes(value.page_scope))
  ) {
    throw new Error('runtime case is invalid');
  }
  assertSha(value.source_sha256, `source hash for ${value.case_id}`);
  if (sha256(value.source) !== value.source_sha256) {
    throw new Error(`runtime case source hash does not match: ${value.case_id}`);
  }
  return value;
}

function validateCapture(value, casesById) {
  if (
    value?.schema !== CAPTURE_SCHEMA ||
    !['captured', 'render-failed'].includes(value.capture_status) ||
    value.site !== 'sandbox-for-codex' ||
    value.domain !== 'sandbox-for-codex.wikidot.com' ||
    value.mutated !== true ||
    value.authenticated_capture !== false ||
    !Number.isSafeInteger(value.page_identity) ||
    typeof value.captured_at !== 'string' ||
    typeof value.saved_source !== 'string'
  ) {
    throw new Error('saved-page capture is invalid');
  }
  assertSha(value.saved_source_sha256, `saved source hash for ${value.page_plan?.slug}`);
  if (sha256(value.saved_source) !== value.saved_source_sha256) {
    throw new Error(`saved source hash does not match: ${value.page_plan?.slug}`);
  }
  const page = value.page_plan;
  if (
    page?.schema !== 'wikijump_syntax_differential.wikidot_page_plan.v1' ||
    typeof page.slug !== 'string' ||
    !/^run-owned:ftml-diff-\d{8}-\d{3}$/u.test(page.slug) ||
    typeof page.title !== 'string' ||
    typeof page.source !== 'string' ||
    !Array.isArray(page.cases)
  ) {
    throw new Error('saved-page plan is invalid');
  }
  assertSha(page.source_sha256, `requested source hash for ${page.slug}`);
  if (sha256(page.source) !== page.source_sha256) {
    throw new Error(`requested source hash does not match: ${page.slug}`);
  }
  if (!value.source_normalized && value.saved_source_sha256 !== page.source_sha256) {
    throw new Error(`unnormalized saved source changed: ${page.slug}`);
  }
  for (const marker of page.cases) {
    const runtimeCase = casesById.get(marker.case_id);
    if (!runtimeCase) continue;
    assertSha(marker.source_sha256, `page case source hash for ${marker.case_id}`);
    if (marker.source_sha256 !== runtimeCase.source_sha256) {
      throw new Error(`page case source identity changed: ${marker.case_id}`);
    }
    if (runtimeCase.page_scope === 'isolated') {
      if (page.cases.length !== 1) {
        throw new Error(`isolated runtime case shares a page: ${marker.case_id}`);
      }
      if (
        marker.page_scope !== 'isolated' ||
        marker.marker_begin != null ||
        marker.marker_end != null ||
        page.source !== runtimeCase.source
      ) {
        throw new Error(`isolated runtime case is not sentinel-free: ${marker.case_id}`);
      }
      continue;
    }
    for (const field of ['marker_begin', 'marker_end']) {
      if (typeof marker[field] !== 'string' || !marker[field].startsWith('WJDIFF_')) {
        throw new Error(`page marker is invalid: ${marker.case_id}`);
      }
    }
  }
  if (value.capture_status === 'captured' && typeof value.page_content_html !== 'string') {
    throw new Error(`captured page has no HTML: ${page.slug}`);
  }
  if (value.capture_status === 'captured') {
    assertSha(value.page_content_html_sha256, `captured HTML hash for ${page.slug}`);
    if (sha256(value.page_content_html) !== value.page_content_html_sha256) {
      throw new Error(`captured HTML hash does not match: ${page.slug}`);
    }
  }
  return value;
}

function validateExternalReference(value, casesById) {
  validateWikidotReference(value);
  const caseId = value.syntax_case.case_id;
  const runtimeCase = casesById.get(caseId);
  if (!runtimeCase) throw new Error(`external reference names an unknown runtime case: ${caseId}`);
  if (
    value.syntax_case.local_execution_tier !== 'ftml' ||
    value.syntax_case.source !== runtimeCase.source ||
    value.source_sha256 !== runtimeCase.source_sha256
  ) {
    throw new Error(`external reference does not reclassify the exact runtime source: ${caseId}`);
  }
  return value;
}

export function selectLatestSuccessfulCaptures(cases, captureFiles, externalReferences = []) {
  const validatedCases = cases.map(validateCase);
  const casesById = new Map(validatedCases.map((value) => [value.case_id, value]));
  if (casesById.size !== validatedCases.length) throw new Error('runtime case IDs are not unique');
  const external = new Map();
  for (const value of externalReferences) {
    const reference = validateExternalReference(value, casesById);
    if (external.has(reference.syntax_case.case_id)) {
      throw new Error(`external reference case is duplicated: ${reference.syntax_case.case_id}`);
    }
    external.set(reference.syntax_case.case_id, reference);
  }
  const selected = new Map();
  const attempts = new Map(validatedCases.map((value) => [value.case_id, []]));
  const captures = captureFiles.flatMap((file, fileIndex) =>
    file.captures.map((rawCapture, lineIndex) => {
      const capture = validateCapture(rawCapture, casesById);
      if (Number.isNaN(Date.parse(capture.captured_at))) {
        throw new Error(`saved-page capture time is invalid: ${capture.page_plan.slug}`);
      }
      return {capture, capturedAt: Date.parse(capture.captured_at), file, fileIndex, lineIndex};
    }),
  ).sort((left, right) => {
    const time = left.capturedAt - right.capturedAt;
    return time || left.fileIndex - right.fileIndex || left.lineIndex - right.lineIndex;
  });
  for (const {capture, file, fileIndex, lineIndex} of captures) {
    let fragments = null;
    let extractionError = null;
    if (capture.capture_status === 'captured') {
      try {
        fragments = extractMarkedFragments(capture.page_content_html, capture.page_plan);
      } catch (error) {
        extractionError = error.message;
      }
    }
    for (const marker of capture.page_plan.cases) {
      if (!casesById.has(marker.case_id) || external.has(marker.case_id)) continue;
      const attempt = {
        capture_file: file.path,
        line: lineIndex + 1,
        slug: capture.page_plan.slug,
        capture_status: capture.capture_status,
        extraction_error: extractionError,
      };
      attempts.get(marker.case_id).push(attempt);
      const fragment = fragments?.get(marker.case_id);
      if (fragment == null) continue;
      selected.set(marker.case_id, {
        case_id: marker.case_id,
        wikidot_html: fragment,
        capture,
        capture_file: file.path,
        capture_file_index: fileIndex,
        capture_line: lineIndex + 1,
      });
    }
  }
  const acquisitionFailed = validatedCases
    .filter((value) => !external.has(value.case_id) && !selected.has(value.case_id))
    .map((value) => ({
      case_id: value.case_id,
      status: 'acquisition-failed',
      attempts: attempts.get(value.case_id),
    }));
  return {cases: validatedCases, casesById, selected, external, acquisitionFailed};
}

export function externalStateReasons(source) {
  const reasons = [];
  if (/\[\[include\s+:[^:\]\s]+:/iu.test(source)) {
    reasons.push('cross-site-include-state');
  } else if (/\[\[include(?:\s|\])/iu.test(source)) {
    reasons.push('include-target-state');
  }
  if (/\[\[\[/u.test(source)) reasons.push('page-existence-state');
  if (/\[\[\*?user(?:\s|\])/iu.test(source)) reasons.push('user-identity-state');
  const modules = [...source.matchAll(/\[\[\s*module\s+([A-Za-z][A-Za-z0-9_-]*)/giu)]
    .map((match) => match[1].toLowerCase())
    .filter((name) => name !== 'css');
  if (modules.length > 0) reasons.push(`module-state:${[...new Set(modules)].sort().join(',')}`);
  return reasons;
}

function attribute(node, name) {
  return node.attrs?.find((entry) => entry.name === name)?.value ?? null;
}

function hasClass(node, className) {
  return attribute(node, 'class')?.split(/\s+/u).includes(className) ?? false;
}

function containsHtmlBlockIframe(nodes) {
  return nodes.some((node) =>
    (node.type === 'element' && node.name === 'iframe' && hasClass(node, 'html-block-iframe')) ||
    containsHtmlBlockIframe(node.children ?? [])
  );
}

function isTabviewRoot(node) {
  return node?.type === 'element' && node.name === 'div' && hasClass(node, 'yui-navset');
}

function tabviewId(node) {
  if (!isTabviewRoot(node)) return null;
  const id = attribute(node, 'id');
  return /^wiki-tabview-[0-9a-f]{32}$/u.test(id ?? '') ? id : null;
}

function knownTabviewLoader(node) {
  if (node.type !== 'element' || node.name !== 'script' || node.children.length !== 0) {
    return false;
  }
  const attrs = new Map(node.attrs.map((entry) => [entry.name, entry.value]));
  if (
    attrs.size !== 2 ||
    attrs.get('type') !== 'text/javascript' ||
    typeof attrs.get('src') !== 'string'
  ) {
    return false;
  }
  let url;
  try {
    url = new URL(attrs.get('src'));
  } catch {
    return false;
  }
  return (
    ['http:', 'https:'].includes(url.protocol) &&
    url.hostname === 'd3g0gp89917ko0.cloudfront.net' &&
    /^\/v--[0-9a-f]+\/common--javascript\/yahooui\/tabview-min\.js$/u.test(url.pathname) &&
    url.search === '' &&
    url.hash === ''
  );
}

function knownTabviewInitializer(node) {
  if (
    node.type !== 'element' ||
    node.name !== 'script' ||
    node.children.length !== 1 ||
    node.children[0].type !== 'text'
  ) {
    return null;
  }
  const attrs = new Map(node.attrs.map((entry) => [entry.name, entry.value]));
  if (attrs.size !== 1 || attrs.get('type') !== 'text/javascript') return null;
  const match = /^\s*\/\/<!\[CDATA\[\s*OZONE\.dom\.onDomReady\(function\(\)\{\s*var tabView(?<nonce>[0-9a-f]{32}) = new YAHOO\.widget\.TabView\('wiki-tabview-\k<nonce>'\);\s*\}, "dummy-ondomready-block"\);\s*\/\/\]\]>\s*$/u.exec(
    node.children[0].value,
  );
  if (!match) return null;
  return `wiki-tabview-${match.groups.nonce}`;
}

function knownWikijumpTabviewPlaceholder(node) {
  if (
    node.type === 'comment' &&
    node.value.trim() === 'Wikidot tabview bootstrap omitted'
  ) {
    return 'placeholder';
  }
  if (node.type !== 'element' || node.name !== 'script' || node.children.length !== 0) {
    return null;
  }
  const attrs = new Map(node.attrs.map((entry) => [entry.name, entry.value]));
  return attrs.size === 1 && attrs.get('type') === 'text/javascript'
    ? 'inert-script'
    : null;
}

function collectTabviewIds(nodes, ids = new Set()) {
  for (const node of nodes) {
    const id = tabviewId(node);
    if (id) ids.add(id);
    collectTabviewIds(node.children ?? [], ids);
  }
  return ids;
}

function tabviewProjection(dom) {
  const ids = collectTabviewIds(dom);
  const idTokens = new Map();
  const initialized = new Set();
  const transport = {loaders: 0, initializers: 0, placeholders: 0, inert_scripts: 0};
  let invalidTransport = false;

  const projectNodes = (nodes) => {
    const projected = [];
    for (let index = 0; index < nodes.length; index += 1) {
      const node = nodes[index];
      const nextIsTabview = tabviewId(nodes[index + 1]) !== null;
      const previousTabviewId = tabviewId(nodes[index - 1]);
      const previousIsTabview = previousTabviewId !== null;
      if (knownTabviewLoader(node) && nextIsTabview) {
        transport.loaders += 1;
        continue;
      }
      const initializerId = knownTabviewInitializer(node);
      if (initializerId) {
        if (!ids.has(initializerId) || initialized.has(initializerId)) {
          invalidTransport = true;
        } else {
          initialized.add(initializerId);
          transport.initializers += 1;
          continue;
        }
      }
      const placeholder = knownWikijumpTabviewPlaceholder(node);
      if (placeholder && (nextIsTabview || previousIsTabview)) {
        transport[placeholder === 'placeholder' ? 'placeholders' : 'inert_scripts'] += 1;
        continue;
      }
      if (node.name === 'script' || node.type === 'comment') {
        invalidTransport = true;
      }

      if (node.type !== 'element') {
        projected.push(node);
        continue;
      }
      const id = tabviewId(node);
      const attrs = node.attrs.map((entry) => {
        if (entry.name !== 'id' || !id) return entry;
        if (!idTokens.has(id)) {
          idTokens.set(id, `wiki-tabview-instance-${idTokens.size}`);
        }
        return {...entry, value: idTokens.get(id)};
      });
      projected.push({
        ...node,
        attrs,
        children: projectNodes(node.children),
      });
    }
    return projected;
  };

  const projected = projectNodes(dom);
  const roots = [];
  const collectRoots = (nodes, insideTabview = false) => {
    for (const node of nodes) {
      const tabview = isTabviewRoot(node);
      if (tabview && !insideTabview) {
        roots.push(node);
        continue;
      }
      collectRoots(node.children ?? [], insideTabview || tabview);
    }
  };
  collectRoots(projected);
  return {roots, tabviewCount: ids.size, transport, invalidTransport};
}

function tabviewTransportStatus(wikidot, wikijump) {
  if (wikidot.invalidTransport || wikijump.invalidTransport) return 'mismatch';
  const completeYui = (value) =>
    value.transport.loaders === value.tabviewCount &&
    value.transport.initializers === value.tabviewCount &&
    value.transport.placeholders === 0 &&
    value.transport.inert_scripts === 0;
  const completePlaceholder = (value) =>
    value.transport.loaders === 0 &&
    value.transport.initializers === 0 &&
    value.transport.placeholders + value.transport.inert_scripts === value.tabviewCount;
  if (completeYui(wikidot) && completePlaceholder(wikijump)) {
    return 'expected-platform-substitution';
  }
  if (
    (completeYui(wikidot) && completeYui(wikijump)) ||
    (completePlaceholder(wikidot) && completePlaceholder(wikijump))
  ) {
    return 'match';
  }
  return 'mismatch';
}

export function compareRuntimeFragment(
  runtimeCase,
  wikidotHtml,
  wikijumpHtml,
  {pageSlug = null, wikijumpIdentityHtml = null, htmlBlockBinding = null} = {},
) {
  const wikidotDom = canonicalDom(wikidotHtml);
  const wikijumpDom = canonicalDom(wikijumpHtml);
  const wikidotText = visibleText(wikidotHtml);
  const wikijumpText = visibleText(wikijumpHtml);
  const domMatches = JSON.stringify(wikidotDom) === JSON.stringify(wikijumpDom);
  const textMatches = wikidotText === wikijumpText;
  const hasHtmlBlocks =
    /\[\[\s*html(?:\s|\])/iu.test(runtimeCase.source) ||
    htmlBlockBinding != null ||
    containsHtmlBlockIframe(wikidotDom) ||
    containsHtmlBlockIframe(wikijumpDom);
  const wikidotHtmlBlocks = hasHtmlBlocks
    ? projectRuntimeHtmlBlocks(wikidotDom, {side: 'wikidot', pageSlug})
    : null;
  const wikijumpHtmlBlocks = hasHtmlBlocks && wikijumpIdentityHtml != null
    ? projectRuntimeHtmlBlocks(canonicalDom(wikijumpIdentityHtml), {
        side: 'wikijump',
        pageSlug,
      })
    : null;
  const htmlBlockPayloadsMatch =
    hasHtmlBlocks &&
    htmlBlockBinding?.status === 'tracked' &&
    wikidotHtmlBlocks != null &&
    wikijumpHtmlBlocks != null &&
    !wikidotHtmlBlocks.invalid &&
    !wikijumpHtmlBlocks.invalid &&
    JSON.stringify(wikidotHtmlBlocks.blocks.map(({sha1: digest}) => digest)) ===
      JSON.stringify(wikijumpHtmlBlocks.blocks.map(({sha1: digest}) => digest));
  const htmlBlockProjectedDomMatches =
    htmlBlockPayloadsMatch &&
    JSON.stringify(wikidotHtmlBlocks.dom) === JSON.stringify(wikijumpHtmlBlocks.dom);
  const effectiveWikidotDom = htmlBlockPayloadsMatch ? wikidotHtmlBlocks.dom : wikidotDom;
  const effectiveWikijumpDom = htmlBlockPayloadsMatch ? wikijumpHtmlBlocks.dom : wikijumpDom;
  const effectiveDomMatches = hasHtmlBlocks
    ? htmlBlockProjectedDomMatches
    : domMatches;
  const htmlBlockChecks = hasHtmlBlocks
    ? {
        html_block_contract: {
          status: htmlBlockProjectedDomMatches ? 'match' : 'mismatch',
          binding: htmlBlockBinding,
          wikidot: wikidotHtmlBlocks == null
            ? null
            : {invalid: wikidotHtmlBlocks.invalid, blocks: wikidotHtmlBlocks.blocks},
          wikijump: wikijumpHtmlBlocks == null
            ? null
            : {invalid: wikijumpHtmlBlocks.invalid, blocks: wikijumpHtmlBlocks.blocks},
        },
      }
    : {};
  const hasTabview = /\[\[(?:tabs|tabview)(?:\s|\])/iu.test(runtimeCase.source);
  const wikidotTabview = hasTabview ? tabviewProjection(effectiveWikidotDom) : null;
  const wikijumpTabview = hasTabview ? tabviewProjection(effectiveWikijumpDom) : null;
  const tabviewStaticMatches = hasTabview &&
    wikidotTabview.roots.length > 0 &&
    JSON.stringify(wikidotTabview.roots) === JSON.stringify(wikijumpTabview.roots);
  const tabviewTransport = hasTabview
    ? tabviewTransportStatus(wikidotTabview, wikijumpTabview)
    : null;
  const tabviewChecks = hasTabview
    ? {
        tabview_static_contract: {
          status: tabviewStaticMatches ? 'match' : 'mismatch',
          tabview_count: wikidotTabview.tabviewCount,
        },
        tabview_bootstrap_transport: {
          status: tabviewTransport,
          wikidot: wikidotTabview.transport,
          wikijump: wikijumpTabview.transport,
        },
        tabview_activation_contract: {status: 'not-run'},
      }
    : {};
  if (
    hasTabview &&
    tabviewStaticMatches &&
    (!hasHtmlBlocks || htmlBlockProjectedDomMatches) &&
    textMatches &&
    ['match', 'expected-platform-substitution'].includes(tabviewTransport)
  ) {
    return {
      case_id: runtimeCase.case_id,
      status: 'static-match-browser-required',
      checks: {
        dom_tree: {status: domMatches ? 'match' : 'mismatch'},
        visible_text: {status: 'match', wikidot: wikidotText, wikijump: wikijumpText},
        ...htmlBlockChecks,
        ...tabviewChecks,
      },
      diagnostic: {wikidot_html: wikidotHtml, wikijump_html: wikijumpHtml},
    };
  }
  if (effectiveDomMatches && textMatches) {
    return {
      case_id: runtimeCase.case_id,
      status: 'match',
      checks: {
        dom_tree: {status: domMatches ? 'match' : 'mismatch'},
        visible_text: {status: 'match', wikidot: wikidotText, wikijump: wikijumpText},
        ...htmlBlockChecks,
        ...tabviewChecks,
      },
    };
  }
  const stateReasons = externalStateReasons(runtimeCase.source);
  return {
    case_id: runtimeCase.case_id,
    status: 'true-mismatch',
    suspected_state_preconditions: stateReasons,
    checks: {
      dom_tree: {status: domMatches ? 'match' : 'mismatch'},
      visible_text: {
        status: textMatches ? 'match' : 'mismatch',
        wikidot: wikidotText,
        wikijump: wikijumpText,
      },
      ...htmlBlockChecks,
      ...tabviewChecks,
    },
    diagnostic: {wikidot_html: wikidotHtml, wikijump_html: wikijumpHtml},
  };
}

function selectedPages(selection) {
  const pages = new Map();
  for (const reference of selection.selected.values()) {
    const key = `${reference.capture_file_index}:${reference.capture_line}`;
    const existing = pages.get(key) ?? {
      capture: reference.capture,
      capture_file: reference.capture_file,
      capture_line: reference.capture_line,
      case_ids: [],
    };
    existing.case_ids.push(reference.case_id);
    pages.set(key, existing);
  }
  return [...pages.values()].sort((left, right) => {
    const time = Date.parse(left.capture.captured_at) - Date.parse(right.capture.captured_at);
    return time || left.capture_file.localeCompare(right.capture_file) || left.capture_line - right.capture_line;
  });
}

export async function runGenericRuntimeDifferential({
  cases,
  captureFiles,
  externalReferences,
  runtimeIdentity,
  adapter,
  inputIdentities = {},
}) {
  const identity = validateRuntimeIdentity(runtimeIdentity);
  const selection = selectLatestSuccessfulCaptures(cases, captureFiles, externalReferences);
  const comparisons = [
    ...[...selection.external].map(([caseId, reference]) => ({
      case_id: caseId,
      status: 'external-reference',
      observation_tier: reference.syntax_case.wikidot_observation_tier,
      raw_html_sha256: reference.raw_html_sha256,
    })),
    ...selection.acquisitionFailed,
  ];
  const pageReceipts = [];
  for (const page of selectedPages(selection)) {
    const capture = page.capture;
    const pageComparisons = [];
    try {
      const receipt = await adapter.withCompiledPage(
        {
          slug: capture.page_plan.slug,
          title: capture.page_plan.title,
          source: capture.saved_source,
          source_sha256: capture.saved_source_sha256,
        },
        async (compiledBodyHtml, htmlBlockEvidence = {iframe_count: 0, blocks: []}) => {
          const boundHtmlBlocks = bindLocalHtmlBlockPayloads(
            compiledBodyHtml,
            htmlBlockEvidence.blocks,
          );
          let fragments = null;
          let identityFragments = null;
          try {
            fragments = extractMarkedFragments(compiledBodyHtml, capture.page_plan);
            identityFragments = extractMarkedFragments(
              boundHtmlBlocks.html,
              capture.page_plan,
            );
          } catch {
            // A syntax case can consume or suppress its own sentinel without
            // invalidating later, independently extractable cases on the page.
          }
          for (const caseId of page.case_ids) {
            const marker = capture.page_plan.cases.find((value) => value.case_id === caseId);
            let localHtml = fragments?.get(caseId);
            let localIdentityHtml = identityFragments?.get(caseId);
            try {
              if (localHtml == null) {
                localHtml = extractMarkedFragments(compiledBodyHtml, {cases: [marker]}).get(caseId);
              }
              if (localIdentityHtml == null) {
                localIdentityHtml = extractMarkedFragments(
                  boundHtmlBlocks.html,
                  {cases: [marker]},
                ).get(caseId);
              }
              if (localHtml == null) throw new Error(`local marker extraction failed: ${caseId}`);
              if (localIdentityHtml == null) {
                throw new Error(`local HTML block identity extraction failed: ${caseId}`);
              }
            } catch (error) {
              pageComparisons.push({
                case_id: caseId,
                status: 'runtime-error',
                diagnostic: {
                  slug: capture.page_plan.slug,
                  error: error instanceof Error ? error.message : String(error),
                },
              });
              continue;
            }
            const reference = selection.selected.get(caseId);
            const runtimeCase = selection.casesById.get(caseId);
            const localBlockProjection = projectRuntimeHtmlBlocks(
              canonicalDom(localIdentityHtml),
              {side: 'wikijump', pageSlug: capture.page_plan.slug},
            );
            const hasCaseHtmlBlocks =
              /\[\[\s*html(?:\s|\])/iu.test(runtimeCase.source) ||
              countLocalHtmlBlockHandles(localHtml) > 0 ||
              reference.wikidot_html.includes('html-block-iframe');
            const caseBlocks = localBlockProjection.blocks.map((block) =>
              htmlBlockEvidence.blocks[block.stored_index - 1]
            ).filter(Boolean);
            const caseBinding = hasCaseHtmlBlocks
              ? {
                  status: boundHtmlBlocks.binding.status,
                  iframe_count: countLocalHtmlBlockHandles(localHtml),
                  stored_block_count: caseBlocks.length,
                  page_iframe_count: boundHtmlBlocks.binding.iframe_count,
                  page_stored_block_count: boundHtmlBlocks.binding.stored_block_count,
                  blocks: caseBlocks,
                }
              : null;
            const comparison = compareRuntimeFragment(
              runtimeCase,
              reference.wikidot_html,
              localHtml,
              {
                pageSlug: capture.page_plan.slug,
                wikijumpIdentityHtml: localIdentityHtml,
                htmlBlockBinding: caseBinding,
              },
            );
            pageComparisons.push({
              ...comparison,
              identities: {
                wikidot_html_sha256: sha256(reference.wikidot_html),
                wikijump_html_sha256: sha256(localHtml),
                capture_file: reference.capture_file,
                capture_line: reference.capture_line,
                page_identity: reference.capture.page_identity,
                saved_source_sha256: reference.capture.saved_source_sha256,
              },
            });
          }
        }
      );
      pageReceipts.push(receipt);
      comparisons.push(...pageComparisons);
    } catch (error) {
      if (error instanceof RuntimeCleanupError) throw error;
      for (const caseId of page.case_ids) {
        comparisons.push({
          case_id: caseId,
          status: 'runtime-error',
          diagnostic: {
            slug: capture.page_plan.slug,
            error: error instanceof Error ? error.message : String(error),
          },
        });
      }
    }
  }
  comparisons.sort((left, right) => left.case_id.localeCompare(right.case_id));
  const count = (status) => comparisons.filter((value) => value.status === status).length;
  const summary = {
    total: selection.cases.length,
    compared:
      count('match') +
      count('static-match-browser-required') +
      count('state-precondition-mismatch') +
      count('true-mismatch'),
    match: count('match'),
    static_match_browser_required: count('static-match-browser-required'),
    external_reference: count('external-reference'),
    acquisition_failed: count('acquisition-failed'),
    state_precondition_mismatch: count('state-precondition-mismatch'),
    true_mismatch: count('true-mismatch'),
    runtime_error: count('runtime-error'),
  };
  const status =
    summary.acquisition_failed === 0 &&
    summary.static_match_browser_required === 0 &&
    summary.state_precondition_mismatch === 0 &&
    summary.true_mismatch === 0 &&
    summary.runtime_error === 0
      ? 'pass'
      : summary.true_mismatch > 0 || summary.runtime_error > 0
        ? 'fail'
        : 'incomplete';
  return {
    schema: REPORT_SCHEMA,
    status,
    runtime_identity: identity,
    input_identities: inputIdentities,
    summary,
    comparisons,
    page_receipts: pageReceipts,
  };
}

export class DeepwellRpcAdapter {
  constructor({
    rpcUrl,
    textBlockBaseUrl,
    siteSlug,
    administratorEmail,
    administratorPassword,
    fetchImpl = fetch,
    textBlockFetchImpl = fetch,
  }) {
    if (siteSlug !== 'sandbox-for-codex') {
      throw new Error('Deepwell RPC adapter accepts only sandbox-for-codex');
    }
    const url = new URL(rpcUrl);
    if (
      url.protocol !== 'http:' ||
      !['127.0.0.1', '[::1]', 'localhost'].includes(url.hostname) ||
      url.pathname !== '/jsonrpc' ||
      url.username ||
      url.password ||
      url.search ||
      url.hash
    ) {
      throw new Error('Deepwell RPC URL must be one loopback HTTP /jsonrpc endpoint');
    }
    const blockUrl = new URL(textBlockBaseUrl);
    if (
      blockUrl.protocol !== 'http:' ||
      !['127.0.0.1', '[::1]', 'localhost'].includes(blockUrl.hostname) ||
      blockUrl.pathname !== '/deepwell-text-blocks/' ||
      blockUrl.username ||
      blockUrl.password ||
      blockUrl.search ||
      blockUrl.hash
    ) {
      throw new Error('text block URL must be one loopback HTTP deepwell-text-blocks bucket');
    }
    this.rpcUrl = url.href;
    this.textBlockBaseUrl = blockUrl.href;
    this.siteSlug = siteSlug;
    this.administratorEmail = administratorEmail;
    this.administratorPassword = administratorPassword;
    this.fetchImpl = fetchImpl;
    this.textBlockFetchImpl = textBlockFetchImpl;
    this.nextId = 1;
    this.connection = null;
  }

  async rpc(method, params = {}, headers = {}) {
    const response = await this.fetchImpl(this.rpcUrl, {
      method: 'POST',
      headers: {'content-type': 'application/json', ...headers},
      body: JSON.stringify({jsonrpc: '2.0', id: this.nextId++, method, params}),
      signal: AbortSignal.timeout(300_000),
    });
    const body = await response.json();
    if (!response.ok || body.error) {
      throw new Error(`${method}: ${JSON.stringify(body.error ?? body)}`);
    }
    return body.result;
  }

  async connect() {
    if (this.connection) return this.connection;
    await this.rpc('ping');
    const site = await this.rpc('site_get', {site: this.siteSlug});
    const login = await this.rpc('login', {
      name_or_email: this.administratorEmail,
      password: this.administratorPassword,
      ip_address: '127.0.0.1',
      user_agent: 'generic-runtime-differential',
    });
    const administrator = await this.rpc('user_get', {user: 'administrator'});
    if (!site?.site_id || !login?.session_token || !administrator?.user_id) {
      throw new Error('Deepwell seeded runtime identity is incomplete');
    }
    this.connection = {
      siteId: site.site_id,
      sessionToken: login.session_token,
      userId: administrator.user_id,
    };
    return this.connection;
  }

  context(slug) {
    return {
      'X-Deepwell-Session-Token': this.connection.sessionToken,
      'X-Deepwell-Site-Id': String(this.connection.siteId),
      'X-Deepwell-Page': slug,
    };
  }

  async getPage(slug) {
    return await this.rpc('page_get', {
      site_id: this.connection.siteId,
      page: slug,
      details: {wikitext: true, compiled: true},
    });
  }

  async getHtmlBlockIndex(pageId, index) {
    return await this.rpc('text_block_get_index', {
      site_id: this.connection.siteId,
      page_id: pageId,
      block_type: 'html',
      index,
      name: null,
      session_token: this.connection.sessionToken,
    });
  }

  textBlockObjectUrl(filename) {
    return new URL(encodeURIComponent(filename), this.textBlockBaseUrl).href;
  }

  async readHtmlBlocks(pageId, compiledBodyHtml) {
    const iframeCount = countLocalHtmlBlockHandles(compiledBodyHtml);
    const blocks = [];
    const scanLimit = Math.min(iframeCount + 1, 32_767);
    for (let index = 1; index <= scanLimit; index += 1) {
      const block = await this.getHtmlBlockIndex(pageId, index);
      if (block == null) break;
      const expectedFilename = `${pageId}_html_${index}`;
      if (block.index !== index || block.s3_filename !== expectedFilename) {
        throw new Error(`local HTML block identity changed at index ${index}`);
      }
      const objectUrl = this.textBlockObjectUrl(block.s3_filename);
      const response = await this.textBlockFetchImpl(objectUrl, {
        signal: AbortSignal.timeout(30_000),
      });
      if (!response.ok) {
        throw new Error(`local HTML block ${index} returned HTTP ${response.status}`);
      }
      const bytes = Buffer.from(await response.arrayBuffer());
      blocks.push({
        index,
        s3_filename: block.s3_filename,
        bytes: bytes.length,
        sha1: sha1(bytes),
        sha256: sha256(bytes),
      });
    }
    return {
      iframe_count: iframeCount,
      blocks,
    };
  }

  async assertHtmlBlocksRemoved(blocks) {
    for (const block of blocks) {
      const response = await this.textBlockFetchImpl(
        this.textBlockObjectUrl(block.s3_filename),
        {signal: AbortSignal.timeout(30_000)},
      );
      if (response.status !== 404) {
        throw new Error(
          `local HTML block remained after cleanup: ${block.s3_filename} returned HTTP ${response.status}`,
        );
      }
    }
  }

  async removeCreatedPage(page, created, inspected, htmlBlocks = []) {
    let latest = inspected;
    if (!latest) {
      try {
        latest = await this.getPage(page.slug);
      } catch {
        latest = null;
      }
    }
    const pageId = latest?.page_id ?? created?.page_id;
    if (!Number.isSafeInteger(pageId)) {
      throw new Error(`local runtime cleanup found no page identity: ${page.slug}`);
    }
    await this.rpc(
      'page_delete',
      {
        site_id: this.connection.siteId,
        page: pageId,
        last_revision_id: latest?.revision_id ?? created?.revision_id,
        revision_comments: 'generic runtime differential cleanup',
        user_id: this.connection.userId,
        ip_address: '127.0.0.1',
      },
      this.context(page.slug),
    );
    if (await this.getPage(page.slug)) {
      throw new Error(`local runtime page remained after cleanup: ${page.slug}`);
    }
    await this.assertHtmlBlocksRemoved(htmlBlocks);
    return {
      slug: page.slug,
      page_id: pageId,
      status: 'removed',
      html_block_objects_removed: htmlBlocks.length,
    };
  }

  async withCompiledPage(page, inspect) {
    await this.connect();
    if (sha256(page.source) !== page.source_sha256) {
      throw new Error(`local runtime source identity changed: ${page.slug}`);
    }
    if (await this.getPage(page.slug)) throw new Error(`local runtime page already exists: ${page.slug}`);
    let created = null;
    let inspected = null;
    let cleanup = null;
    let htmlBlockEvidence = {iframe_count: 0, blocks: []};
    let operationError = null;
    let cleanupError = null;
    try {
      created = await this.rpc(
        'page_create',
        {
          site_id: this.connection.siteId,
          slug: page.slug,
          title: page.title,
          wikitext: page.source,
          layout: 'wikidot',
          user_id: this.connection.userId,
          ip_address: '127.0.0.1',
          tags: [],
          revision_comments: 'generic runtime differential fixture',
        },
        this.context(page.slug),
      );
      if (!Number.isSafeInteger(created?.page_id) || !Number.isSafeInteger(created?.revision_id)) {
        throw new Error(`local runtime page creation returned no identity: ${page.slug}`);
      }
      inspected = await this.getPage(page.slug);
      if (
        !inspected ||
        inspected.wikitext !== page.source ||
        typeof inspected.compiled_body_html !== 'string'
      ) {
        throw new Error(`local runtime page did not round-trip: ${page.slug}`);
      }
      htmlBlockEvidence = await this.readHtmlBlocks(
        inspected.page_id,
        inspected.compiled_body_html,
      );
      await inspect(inspected.compiled_body_html, htmlBlockEvidence);
    } catch (error) {
      operationError = error;
    } finally {
      let cleanupTarget = inspected;
      if (!cleanupTarget) {
        try {
          cleanupTarget = await this.getPage(page.slug);
        } catch {
          cleanupTarget = null;
        }
      }
      if (created || cleanupTarget) {
        try {
          cleanup = await this.removeCreatedPage(
            page,
            created,
            cleanupTarget,
            htmlBlockEvidence.blocks,
          );
        } catch (error) {
          cleanupError = new RuntimeCleanupError(
            `local runtime cleanup failed for ${page.slug}: ${error instanceof Error ? error.message : String(error)}`,
            {cause: error},
          );
        }
      }
    }
    if (cleanupError) throw cleanupError;
    if (operationError) throw operationError;
    return {
      slug: page.slug,
      source_sha256: page.source_sha256,
      page_id: inspected.page_id,
      revision_id: inspected.revision_id,
      html_blocks: htmlBlockEvidence.blocks,
      cleanup,
    };
  }

  async close() {}
}
