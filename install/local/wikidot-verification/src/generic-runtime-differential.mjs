import {canonicalDom, sha256, validateWikidotReference, visibleText} from './syntax-differential.mjs';
import {
  bindLocalHtmlBlockPayloads,
  countLocalHtmlBlockHandles,
  projectRuntimeHtmlBlocks,
  sha1,
} from './runtime-html-blocks.mjs';
import {validateRuntimeIdentity} from './saved-page-runtime-differential.mjs';
import {validateRuntimeStateFixtureInput} from './runtime-state-fixture.mjs';
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
    !['ftml', 'wikijump-runtime'].includes(value.syntax_case.local_execution_tier) ||
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

function containsElement(nodes, name) {
  return nodes.some((node) =>
    (node.type === 'element' && node.name === name) ||
    containsElement(node.children ?? [], name)
  );
}

function isAcceptedFileTraversalDeviation(runtimeCase, wikidotDom, wikijumpText) {
  const match = runtimeCase.source.match(
    /^\s*\[\[\s*file\s+([^\]\s|]+)(?:\s*\|\s*[^\]]*)?\]\]\s*$/iu,
  );
  if (!match || !match[1].split(/[\\/]/u).includes('..')) return false;
  return wikijumpText.trim() === runtimeCase.source.trim() && containsElement(wikidotDom, 'a');
}

function nodeText(node) {
  if (node.type === 'text') return node.value;
  return (node.children ?? []).map(nodeText).join('');
}

function projectCategoriesDom(dom) {
  const categoryIds = new Map();
  const idCategories = new Map();
  let invalid = false;
  let categoryCount = 0;
  const project = (node) => {
    const children = (node.children ?? []).map(project);
    if (node.type !== 'element' || node.name !== 'div') return {...node, children};
    const elements = children.filter((child) => child.type === 'element');
    if (
      elements.length !== 4 ||
      elements[0].name !== 'h3' ||
      elements[1].name !== 'a' ||
      elements[2].name !== 'div' ||
      elements[3].name !== 'div'
    ) {
      return {...node, children};
    }
    const category = nodeText(elements[0]);
    const toggler = elements[1];
    const pages = elements[2];
    const options = elements[3];
    const togglerId = attribute(toggler, 'id')?.match(/^category-pages-toggler-(\d+)$/u)?.[1];
    const pagesId = attribute(pages, 'id')?.match(/^category-pages-(\d+)$/u)?.[1];
    const optionsId = attribute(options, 'id')?.match(/^category-pages-(\d+)-options$/u)?.[1];
    const onclickId = attribute(toggler, 'onclick')?.match(
      /^WIKIDOT\.modules\.WikiCategoriesModule\.listeners\.toggleListPages\(event, (\d+)\)$/u,
    )?.[1];
    if (
      category.length === 0 ||
      togglerId == null ||
      togglerId !== pagesId ||
      togglerId !== optionsId ||
      togglerId !== onclickId ||
      attribute(toggler, 'href') !== 'javascript:;' ||
      attribute(pages, 'style') !== 'display: none' ||
      attribute(options, 'style') !== 'display: none' ||
      (categoryIds.has(category) && categoryIds.get(category) !== togglerId) ||
      (idCategories.has(togglerId) && idCategories.get(togglerId) !== category)
    ) {
      invalid = true;
      return {...node, children};
    }
    categoryIds.set(category, togglerId);
    idCategories.set(togglerId, category);
    categoryCount += 1;
    const token = `category-id:${category}`;
    const replaceAttribute = (entry) => {
      if (entry.name === 'id') {
        return {
          ...entry,
          value: entry.value
            .replace(`category-pages-toggler-${togglerId}`, `category-pages-toggler-${token}`)
            .replace(`category-pages-${togglerId}`, `category-pages-${token}`),
        };
      }
      if (entry.name === 'onclick') {
        return {...entry, value: entry.value.replace(`event, ${togglerId})`, `event, ${token})`)};
      }
      return entry;
    };
    const projected = children.map((child) => {
      if (![toggler, pages, options].includes(child)) return child;
      return {...child, attrs: child.attrs.map(replaceAttribute)};
    });
    return {...node, children: projected};
  };
  const projected = dom.map(project);
  return {dom: projected, invalid: invalid || categoryCount === 0, category_count: categoryCount};
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
  const hasCategories = /\[\[\s*module\s+categories(?:\s|\])/iu.test(runtimeCase.source);
  const wikidotCategories = hasCategories ? projectCategoriesDom(effectiveWikidotDom) : null;
  const wikijumpCategories = hasCategories ? projectCategoriesDom(effectiveWikijumpDom) : null;
  const categoryProjectedDomMatches =
    hasCategories &&
    !wikidotCategories.invalid &&
    !wikijumpCategories.invalid &&
    JSON.stringify(wikidotCategories.dom) === JSON.stringify(wikijumpCategories.dom);
  const contractDomMatches = hasCategories ? categoryProjectedDomMatches : effectiveDomMatches;
  const categoryChecks = hasCategories
    ? {
        categories_contract: {
          status: categoryProjectedDomMatches ? 'match' : 'mismatch',
          wikidot: {
            invalid: wikidotCategories.invalid,
            category_count: wikidotCategories.category_count,
          },
          wikijump: {
            invalid: wikijumpCategories.invalid,
            category_count: wikijumpCategories.category_count,
          },
        },
      }
    : {};
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
        ...categoryChecks,
        ...tabviewChecks,
      },
      diagnostic: {wikidot_html: wikidotHtml, wikijump_html: wikijumpHtml},
    };
  }
  if (contractDomMatches && textMatches) {
    return {
      case_id: runtimeCase.case_id,
      status: 'match',
      checks: {
        dom_tree: {status: domMatches ? 'match' : 'mismatch'},
        visible_text: {status: 'match', wikidot: wikidotText, wikijump: wikijumpText},
        ...htmlBlockChecks,
        ...categoryChecks,
        ...tabviewChecks,
      },
    };
  }
  if (isAcceptedFileTraversalDeviation(runtimeCase, wikidotDom, wikijumpText)) {
    return {
      case_id: runtimeCase.case_id,
      status: 'accepted-security-deviation',
      deviation: 'file-traversal-target-preserved-literal',
      checks: {
        dom_tree: {status: 'intentionally-different'},
        visible_text: {
          status: textMatches ? 'match' : 'intentionally-different',
          wikidot: wikidotText,
          wikijump: wikijumpText,
        },
      },
      diagnostic: {wikidot_html: wikidotHtml, wikijump_html: wikijumpHtml},
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
      ...categoryChecks,
      ...tabviewChecks,
    },
    diagnostic: {wikidot_html: wikidotHtml, wikijump_html: wikijumpHtml},
  };
}

export async function runGenericRuntimeDifferential({
  cases,
  captureFiles,
  externalReferences,
  runtimeIdentity,
  adapter,
  inputIdentities = {},
  stateFixtures = [],
  disposableRunId = null,
}) {
  const identity = validateRuntimeIdentity(runtimeIdentity);
  const validatedStateFixtures = stateFixtures.map(validateRuntimeStateFixtureInput);
  const stateFixtureReceipts = [];
  for (const stateFixture of validatedStateFixtures) {
    stateFixtureReceipts.push(await adapter.applyStateFixture(stateFixture, disposableRunId));
  }
  const selection = selectLatestSuccessfulCaptures(cases, captureFiles, externalReferences);
  const comparisons = [...selection.acquisitionFailed];
  const pageReceipts = [];
  for (const [caseId, reference] of selection.external) {
    const runtimeCase = selection.casesById.get(caseId);
    const slug = `run-owned:ftml-preview-${runtimeCase.source_sha256.slice(0, 24)}`;
    try {
      const receipt = await adapter.withPreview(
        {
          slug,
          title: reference.syntax_case.title,
          source: runtimeCase.source,
          source_sha256: runtimeCase.source_sha256,
        },
        async (compiledBodyHtml, htmlBlockEvidence = {iframe_count: 0, blocks: []}) => {
          const boundHtmlBlocks = bindLocalHtmlBlockPayloads(
            compiledBodyHtml,
            htmlBlockEvidence.blocks,
          );
          const localBlockProjection = projectRuntimeHtmlBlocks(
            canonicalDom(boundHtmlBlocks.html),
            {side: 'wikijump', pageSlug: slug},
          );
          const hasHtmlBlocks =
            /\[\[\s*html(?:\s|\])/iu.test(runtimeCase.source) ||
            countLocalHtmlBlockHandles(compiledBodyHtml) > 0 ||
            reference.raw_html.includes('html-block-iframe');
          const comparison = compareRuntimeFragment(
            runtimeCase,
            reference.raw_html,
            compiledBodyHtml,
            {
              pageSlug: slug,
              wikijumpIdentityHtml: boundHtmlBlocks.html,
              htmlBlockBinding: hasHtmlBlocks
                ? {
                    status: boundHtmlBlocks.binding.status,
                    iframe_count: countLocalHtmlBlockHandles(compiledBodyHtml),
                    stored_block_count: localBlockProjection.blocks.length,
                    page_iframe_count: boundHtmlBlocks.binding.iframe_count,
                    page_stored_block_count: boundHtmlBlocks.binding.stored_block_count,
                    blocks: localBlockProjection.blocks.map((block) =>
                      htmlBlockEvidence.blocks[block.stored_index - 1]
                    ).filter(Boolean),
                  }
                : null,
            },
          );
          comparisons.push({
            ...comparison,
            identities: {
              wikidot_html_sha256: reference.raw_html_sha256,
              wikijump_html_sha256: sha256(compiledBodyHtml),
              observation_tier: reference.syntax_case.wikidot_observation_tier,
              source_sha256: runtimeCase.source_sha256,
            },
          });
        },
      );
      pageReceipts.push(receipt);
    } catch (error) {
      if (error instanceof RuntimeCleanupError) throw error;
      comparisons.push({
        case_id: caseId,
        status: 'runtime-error',
        diagnostic: {
          slug,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  }
  for (const [caseId, reference] of selection.selected) {
    const runtimeCase = selection.casesById.get(caseId);
    const capture = reference.capture;
    const slug = capture.page_plan.slug;
    try {
      const receipt = await adapter.withCompiledPage(
        {
          slug,
          title: capture.page_plan.title,
          source: runtimeCase.source,
          source_sha256: runtimeCase.source_sha256,
        },
        async (compiledBodyHtml, htmlBlockEvidence = {iframe_count: 0, blocks: []}) => {
          const boundHtmlBlocks = bindLocalHtmlBlockPayloads(
            compiledBodyHtml,
            htmlBlockEvidence.blocks,
          );
          const localBlockProjection = projectRuntimeHtmlBlocks(
            canonicalDom(boundHtmlBlocks.html),
            {side: 'wikijump', pageSlug: capture.page_plan.slug},
          );
          const hasHtmlBlocks =
            /\[\[\s*html(?:\s|\])/iu.test(runtimeCase.source) ||
            countLocalHtmlBlockHandles(compiledBodyHtml) > 0 ||
            reference.wikidot_html.includes('html-block-iframe');
          const comparison = compareRuntimeFragment(
            runtimeCase,
            reference.wikidot_html,
            compiledBodyHtml,
            {
              pageSlug: capture.page_plan.slug,
              wikijumpIdentityHtml: boundHtmlBlocks.html,
              htmlBlockBinding: hasHtmlBlocks
                ? {
                    status: boundHtmlBlocks.binding.status,
                    iframe_count: countLocalHtmlBlockHandles(compiledBodyHtml),
                    stored_block_count: localBlockProjection.blocks.length,
                    page_iframe_count: boundHtmlBlocks.binding.iframe_count,
                    page_stored_block_count: boundHtmlBlocks.binding.stored_block_count,
                    blocks: localBlockProjection.blocks.map((block) =>
                      htmlBlockEvidence.blocks[block.stored_index - 1]
                    ).filter(Boolean),
                  }
                : null,
            },
          );
          comparisons.push({
            ...comparison,
            identities: {
              wikidot_html_sha256: sha256(reference.wikidot_html),
              wikijump_html_sha256: sha256(compiledBodyHtml),
              capture_file: reference.capture_file,
              capture_line: reference.capture_line,
              page_identity: reference.capture.page_identity,
              saved_source_sha256: reference.capture.saved_source_sha256,
              local_execution: 'sentinel-free-singleton',
              wikidot_batch_slug: capture.page_plan.slug,
              wikijump_singleton_slug: slug,
            },
          });
        },
      );
      pageReceipts.push(receipt);
    } catch (error) {
      if (error instanceof RuntimeCleanupError) throw error;
      comparisons.push({
        case_id: caseId,
        status: 'runtime-error',
        diagnostic: {
          slug,
          wikidot_batch_slug: capture.page_plan.slug,
          error: error instanceof Error ? error.message : String(error),
        },
      });
    }
  }
  comparisons.sort((left, right) => left.case_id.localeCompare(right.case_id));
  const count = (status) => comparisons.filter((value) => value.status === status).length;
  const summary = {
    total: selection.cases.length,
    compared:
      count('match') +
      count('static-match-browser-required') +
      count('accepted-security-deviation') +
      count('state-precondition-mismatch') +
      count('true-mismatch'),
    match: count('match'),
    static_match_browser_required: count('static-match-browser-required'),
    accepted_security_deviation: count('accepted-security-deviation'),
    external_reference: selection.external.size,
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
    state_fixture_receipts: stateFixtureReceipts,
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
    this.sites = new Map();
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
    this.sites.set(this.siteSlug, site.site_id);
    return this.connection;
  }

  context(slug, siteId = this.connection.siteId) {
    return {
      'X-Deepwell-Session-Token': this.connection.sessionToken,
      'X-Deepwell-Site-Id': String(siteId),
      'X-Deepwell-Page': slug,
    };
  }

  async siteId(siteSlug) {
    await this.connect();
    const known = this.sites.get(siteSlug);
    if (known != null) return known;
    const site = await this.rpc('site_get', {site: siteSlug});
    if (!Number.isSafeInteger(site?.site_id)) {
      throw new Error(`runtime state fixture site does not exist: ${siteSlug}`);
    }
    this.sites.set(siteSlug, site.site_id);
    return site.site_id;
  }

  async getPageInSite(siteId, slug) {
    return await this.rpc('page_get', {
      site_id: siteId,
      page: slug,
      details: {wikitext: true, compiled: false},
    });
  }

  async createFixturePage(siteId, page) {
    return await this.rpc(
      'page_create',
      {
        site_id: siteId,
        slug: page.slug,
        title: page.title,
        alt_title: null,
        wikitext: page.wikitext,
        layout: 'wikidot',
        user_id: this.connection.userId,
        ip_address: '127.0.0.1',
        tags: [],
        revision_comments: 'generic runtime differential state fixture',
      },
      this.context(page.slug, siteId),
    );
  }

  async deleteFixturePage(siteId, page) {
    await this.rpc(
      'page_delete',
      {
        site_id: siteId,
        page: page.page_id,
        last_revision_id: page.revision_id,
        revision_comments: 'generic runtime differential state fixture',
        user_id: this.connection.userId,
        ip_address: '127.0.0.1',
      },
      this.context(page.slug, siteId),
    );
  }

  async applyStateFixture(input, disposableRunId) {
    if (typeof disposableRunId !== 'string' || !/^runtime-diff-[0-9a-f-]{12}$/u.test(disposableRunId)) {
      throw new Error('runtime state fixtures require the disposable stack controller');
    }
    await this.connect();
    const operations = [];
    const presentPages = [];
    for (const user of input.fixture.wikidot_users) {
      let current = await this.rpc('user_get', {user: user.user_id});
      let action = 'unchanged';
      if (current == null) {
        const imported = await this.rpc('import_wikidot_user', {
          user_id: user.user_id,
          created_at: user.provenance.captured_at,
          fetched_at: user.provenance.captured_at,
          user_type: 'extant',
          name: user.name,
          slug: user.slug,
          avatar_uploaded_blob_id: null,
          real_name: null,
          gender: null,
          birthday: null,
          location: null,
          biography: null,
          website: null,
          karma: 0,
          is_pro: false,
          importing_user_id: this.connection.userId,
          ip_address: '127.0.0.1',
        });
        if (imported?.user_id !== user.user_id) {
          throw new Error(`runtime state fixture could not import Wikidot user: ${user.user_id}`);
        }
        current = await this.rpc('user_get', {user: user.user_id});
        action = 'imported';
      }
      if (
        current?.user_id !== user.user_id ||
        current.user_type !== 'wikidot' ||
        current.name !== user.name ||
        current.slug !== user.slug
      ) {
        throw new Error(`runtime state fixture Wikidot user did not round-trip: ${user.user_id}`);
      }
      operations.push({
        kind: 'wikidot-user',
        user_id: user.user_id,
        name: user.name,
        slug: user.slug,
        provenance_time: user.provenance.captured_at,
        action,
      });
    }
    for (const page of input.fixture.pages) {
      const siteId = await this.siteId(page.site);
      let current = await this.getPageInSite(siteId, page.slug);
      let action;
      if (current == null) {
        await this.createFixturePage(siteId, page);
        action = 'created';
      } else {
        const contentChanged = current.wikitext !== page.wikitext || current.title !== page.title;
        const layoutChanged = current.layout !== 'wikidot';
        if (contentChanged) {
          await this.rpc(
            'page_edit',
            {
              site_id: siteId,
              page: current.page_id,
              last_revision_id: current.revision_id,
              revision_comments: 'generic runtime differential state fixture',
              user_id: this.connection.userId,
              ip_address: '127.0.0.1',
              wikitext: page.wikitext,
              title: page.title,
            },
            this.context(page.slug, siteId),
          );
        }
        if (layoutChanged) {
          await this.rpc(
            'page_set_layout',
            {
              site_id: siteId,
              page_id: current.page_id,
              layout: 'wikidot',
              user_id: this.connection.userId,
              ip_address: '127.0.0.1',
            },
            this.context(page.slug, siteId),
          );
        }
        action = contentChanged || layoutChanged ? 'edited' : 'unchanged';
      }
      current = await this.getPageInSite(siteId, page.slug);
      if (
        current?.wikitext !== page.wikitext ||
        current.title !== page.title ||
        current.layout !== 'wikidot' ||
        sha256(current.wikitext) !== page.source_sha256
      ) {
        throw new Error(`runtime state fixture page did not round-trip: ${page.site}:${page.slug}`);
      }
      presentPages.push({siteId, page: current});
      operations.push({
        kind: 'page',
        site: page.site,
        slug: page.slug,
        source_sha256: page.source_sha256,
        page_id: current.page_id,
        revision_id: current.revision_id,
        action,
      });
    }
    for (const page of input.fixture.absent_pages) {
      const siteId = await this.siteId(page.site);
      const current = await this.getPageInSite(siteId, page.slug);
      if (current != null) await this.deleteFixturePage(siteId, current);
      if (await this.getPageInSite(siteId, page.slug)) {
        throw new Error(`runtime state fixture page remains present: ${page.site}:${page.slug}`);
      }
      operations.push({
        kind: 'absent-page',
        site: page.site,
        slug: page.slug,
        action: current == null ? 'already-absent' : 'deleted',
      });
    }
    for (const category of input.fixture.categories) {
      const siteId = await this.siteId(category.site);
      let current = await this.rpc('category_get', {site: siteId, category: category.slug});
      let action = 'unchanged';
      let seedPage = null;
      if (current == null) {
        const separator = category.slug === '_default' ? '' : `${category.slug}:`;
        const slug = `${separator}run-owned-state-fixture-${disposableRunId}`;
        if (await this.getPageInSite(siteId, slug)) {
          throw new Error(`runtime state fixture category seed page already exists: ${category.site}:${slug}`);
        }
        const seed = {
          slug,
          title: slug,
          wikitext: '',
        };
        const created = await this.createFixturePage(siteId, seed);
        seedPage = await this.getPageInSite(siteId, slug);
        if (!Number.isSafeInteger(created?.page_id) || seedPage == null) {
          throw new Error(`runtime state fixture could not seed category: ${category.site}:${category.slug}`);
        }
        current = await this.rpc('category_get', {site: siteId, category: category.slug});
        action = 'created';
      }
      if (current?.slug !== category.slug) {
        throw new Error(`runtime state fixture category did not round-trip: ${category.site}:${category.slug}`);
      }
      operations.push({
        kind: 'category',
        site: category.site,
        slug: category.slug,
        category_id: current.category_id,
        action,
        ...(seedPage == null
          ? {}
          : {seed_page_id: seedPage.page_id, seed_page_slug: seedPage.slug}),
      });
    }
    for (const {siteId, page} of presentPages.reverse()) {
      await this.rpc(
        'page_rerender',
        {
          site_id: siteId,
          category_id: page.page_category_id,
          page_id: page.page_id,
        },
        this.context(page.slug, siteId),
      );
    }
    return {
      schema: input.fixture.schema,
      path: input.path,
      sha256: input.sha256,
      captured_at: input.fixture.captured_at,
      capture_source: input.fixture.capture_source,
      operations,
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

  async withPreview(page, inspect) {
    await this.connect();
    if (sha256(page.source) !== page.source_sha256) {
      throw new Error(`local runtime source identity changed: ${page.slug}`);
    }
    const preview = await this.rpc(
      'wikidot_page_preview',
      {
        site_id: this.connection.siteId,
        title: page.title,
        wikitext: page.source,
      },
      this.context(''),
    );
    if (!preview || typeof preview.body !== 'string' || !Array.isArray(preview.styles)) {
      throw new Error(`local runtime preview returned no rendered body: ${page.slug}`);
    }
    await inspect(preview.body);
    return {
      source_sha256: page.source_sha256,
      execution_context: 'unsaved-page-preview',
      styles: preview.styles,
      cleanup: {status: 'not-required'},
    };
  }

  async close() {}
}
