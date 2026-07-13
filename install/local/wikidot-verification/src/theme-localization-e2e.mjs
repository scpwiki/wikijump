import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

export const THEME_LOCALIZATION_E2E_SCHEMA = "wikijump_local_lab.theme_localization_e2e_plan.v1";
export const ALLOWED_SITE_SLUG = "scpaiueouiuiuiui";
export const DEFAULT_WIKIDOT_ORIGIN = "http://scpaiueouiuiuiui.wikidot.com";
export const DEFAULT_WIKIJUMP_ORIGIN = "https://scpaiueouiuiuiui.wikijump.localhost:18443";
export const RUN_OWNED_SLUG_PREFIX = "codex-l10n:";
export const LEGACY_RUN_OWNED_SLUG_PREFIX = "theme:codex-l10n-";

export const THEME_CAPTURE_VIEWPORTS = Object.freeze([
  Object.freeze({id: "desktop", width: 1440, height: 1000}),
  Object.freeze({id: "tablet", width: 768, height: 1024}),
  Object.freeze({id: "mobile", width: 390, height: 844}),
]);

export const THEME_PERFORMANCE_GATES = Object.freeze({
  ttfb_ms: Object.freeze({operator: "lte", value: 800}),
  fcp_ms: Object.freeze({operator: "lte", value: 1800}),
  lcp_ms: Object.freeze({operator: "lte", value: 2500}),
  cls: Object.freeze({operator: "lte", value: 0.1}),
  visual_response_ms: Object.freeze({operator: "lte", value: 100}),
  inp_ms: Object.freeze({operator: "lte", value: 200}),
});

const COMMON_STYLE_PROPERTIES = Object.freeze([
  "display",
  "visibility",
  "opacity",
  "position",
  "width",
  "height",
  "color",
  "background-color",
  "background-image",
  "font-family",
  "font-size",
  "line-height",
  "transform",
  "content",
  "--logo",
  "--header-title",
  "--header-subtitle",
]);

export const THEME_STYLE_PROBE_EXPECTATIONS = Object.freeze(["required", "optional", "expected_absent"]);
const THEME_STYLE_PROBE_EXPECTATION_SET = new Set(THEME_STYLE_PROBE_EXPECTATIONS);
const THEME_STYLE_PROPERTY_OPERATORS = new Set(["eq", "one_of"]);

export function themeComputedStyleProperties(contract) {
  const properties = [...(contract?.properties ?? [])];
  for (const probe of contract?.probes ?? []) properties.push(...Object.keys(probe.expected_properties ?? {}));
  return [...new Set(properties)];
}

export function validateThemeComputedStyleContract(contract, {label = "theme computed-style contract"} = {}) {
  if (!contract || !Array.isArray(contract.properties) || contract.properties.length === 0 || !Array.isArray(contract.probes) || contract.probes.length === 0) {
    throw new Error(`${label} must define non-empty properties and probes`);
  }
  if (contract.properties.some((property) => typeof property !== "string" || !property.trim()) || new Set(contract.properties).size !== contract.properties.length) {
    throw new Error(`${label} has invalid or duplicate properties`);
  }
  const ids = new Set();
  for (const probe of contract.probes) {
    if (!probe || typeof probe.id !== "string" || !/^[a-z0-9][a-z0-9_]{0,63}$/u.test(probe.id) || typeof probe.selector !== "string" || !probe.selector.trim()) {
      throw new Error(`${label} has an invalid probe`);
    }
    if (ids.has(probe.id)) throw new Error(`${label} has duplicate probe id: ${probe.id}`);
    ids.add(probe.id);
    if (!THEME_STYLE_PROBE_EXPECTATION_SET.has(probe.expectation)) {
      throw new Error(`${label} probe ${probe.id} has invalid expectation: ${String(probe.expectation)}`);
    }
    if (probe.pseudo !== undefined && (typeof probe.pseudo !== "string" || !/^::[a-z-]+$/u.test(probe.pseudo))) {
      throw new Error(`${label} probe ${probe.id} has an invalid pseudo-element`);
    }
    const expectedProperties = probe.expected_properties;
    if (expectedProperties === undefined) continue;
    if (!expectedProperties || Array.isArray(expectedProperties) || typeof expectedProperties !== "object" || Object.keys(expectedProperties).length === 0) {
      throw new Error(`${label} probe ${probe.id} has invalid expected_properties`);
    }
    if (probe.expectation === "expected_absent") throw new Error(`${label} probe ${probe.id} cannot assert properties while expected absent`);
    for (const [property, specification] of Object.entries(expectedProperties)) {
      if (!/^(?:--)?[a-z][a-z0-9-]*$/u.test(property) || !specification || typeof specification !== "object" || Array.isArray(specification) || !THEME_STYLE_PROPERTY_OPERATORS.has(specification.operator)) {
        throw new Error(`${label} probe ${probe.id} has an invalid expected property: ${property}`);
      }
      const keys = Object.keys(specification).sort();
      if (specification.operator === "eq") {
        if (keys.join(",") !== "operator,value" || typeof specification.value !== "string") throw new Error(`${label} probe ${probe.id} has an invalid eq expectation: ${property}`);
      } else if (keys.join(",") !== "operator,values" || !Array.isArray(specification.values) || specification.values.length === 0 || specification.values.some((value) => typeof value !== "string") || new Set(specification.values).size !== specification.values.length) {
        throw new Error(`${label} probe ${probe.id} has an invalid one_of expectation: ${property}`);
      }
    }
  }
  return contract;
}

const COMMON_PROBES = Object.freeze([
  Object.freeze({id: "header", selector: "#header", expectation: "required"}),
  Object.freeze({id: "side_bar", selector: "#side-bar", expectation: "required"}),
  Object.freeze({id: "main_content", selector: "#main-content", expectation: "required"}),
  Object.freeze({id: "page_content", selector: "#page-content", expectation: "required"}),
  Object.freeze({id: "rate_widget", selector: ".page-rate-widget-box", expectation: "required"}),
  Object.freeze({id: "interwiki_frame", selector: "iframe.scpnet-interwiki-frame", expectation: "optional"}),
]);

export const THEME_LOCALIZATION_TIERS = Object.freeze([
  Object.freeze({
    id: "yossistyle",
    order: 1,
    risk: "baseline",
    article_slug: "theme:yossistyle",
    accepted_source: "translations/jp/en/pages/theme:yossistyle/source.wikidot.txt",
    run_owned_tags: Object.freeze(["テーマ"]),
    minimum_shape: Object.freeze({bytes: 9000, logical_lines: 180, css_modules: 1, code_blocks: 1}),
    required_markers: Object.freeze(["[[module Rate]]", "#header h2 span", "[[collapsible"]),
    dependencies: Object.freeze({components: Object.freeze([]), assets: Object.freeze([]), remote_local_code: Object.freeze([])}),
    computed_style_probes: Object.freeze([
      ...COMMON_PROBES.map((probe) => probe.id === "side_bar" ? Object.freeze({...probe, expected_properties: Object.freeze({"animation-name": Object.freeze({operator: "eq", value: "sbs"}), "animation-duration": Object.freeze({operator: "eq", value: "20000s"}), "animation-iteration-count": Object.freeze({operator: "eq", value: "infinite"})})}) : probe),
      Object.freeze({id: "header_subtitle", selector: "#header h2 span", expectation: "required", expected_properties: Object.freeze({"margin-left": Object.freeze({operator: "eq", value: "1px"})})}),
      Object.freeze({id: "content_link", selector: "#page-content a[href='https://www.youtube.com/watch?v=rRPQs_kM_nw']", expectation: "required", expected_properties: Object.freeze({color: Object.freeze({operator: "eq", value: "rgb(187, 1, 17)"})})}),
      Object.freeze({id: "license_suffix", selector: "#license-area a", pseudo: "::after", expectation: "required", expected_properties: Object.freeze({content: Object.freeze({operator: "eq", value: "\".\""})})}),
      Object.freeze({id: "watchers_button", selector: "#watchers-button", expectation: "optional", expected_properties: Object.freeze({display: Object.freeze({operator: "eq", value: "none"})})}),
      Object.freeze({id: "rate_points", selector: ".page-rate-widget-box .rate-points", expectation: "required", expected_properties: Object.freeze({"text-transform": Object.freeze({operator: "eq", value: "capitalize"})})}),
      Object.freeze({id: "theme_tag", selector: "a[href='/system:page-tags/tag/テーマ#pages']", expectation: "required", expected_properties: Object.freeze({"font-weight": Object.freeze({operator: "one_of", values: Object.freeze(["700", "bold"])})})}),
    ]),
    interactions: Object.freeze([
      Object.freeze({
        id: "collapsible_toggle",
        action: "click",
        target_selectors: Object.freeze(["details.collapsible-block > summary", ".collapsible-block-folded a"]),
        postcondition: "expanded_state_changes",
        gates: Object.freeze(["visual_response_ms", "inp_ms"]),
      }),
    ]),
  }),
  Object.freeze({
    id: "ashes-to-ashes",
    order: 2,
    risk: "dependency-canary",
    article_slug: "theme:ashes-to-ashes",
    accepted_source: "translations/jp/en/pages/theme:ashes-to-ashes/source.wikidot.txt",
    minimum_shape: Object.freeze({bytes: 5500, logical_lines: 200, css_modules: 2, executable_includes: 4, local_resource_references: 4}),
    required_markers: Object.freeze(["component:toggle-sidebar-bhl", "overseer4.webp", "--head-prelude"]),
    dependencies: Object.freeze({
      components: Object.freeze(["component:toggle-sidebar-bhl", "component:centered-header-bhl", "component:image-block"]),
      assets: Object.freeze(["fire.webp", "bamboo.webp", "parchment.webp", "overseer4.webp"]),
      remote_local_code: Object.freeze([]),
    }),
    computed_style_probes: Object.freeze([
      ...COMMON_PROBES,
      Object.freeze({id: "container", selector: "#container", expectation: "required"}),
      Object.freeze({id: "content_before", selector: "#content-wrap", pseudo: "::before", expectation: "required"}),
      Object.freeze({id: "content_after", selector: "#content-wrap", pseudo: "::after", expectation: "required"}),
      Object.freeze({id: "page_title", selector: "#page-title", expectation: "required"}),
      Object.freeze({id: "image_block", selector: ".scp-image-block, .image-block", expectation: "required"}),
    ]),
    interactions: Object.freeze([
      Object.freeze({
        id: "collapsible_toggle",
        action: "click",
        target_selectors: Object.freeze(["details.collapsible-block > summary", ".collapsible-block-folded a"]),
        postcondition: "expanded_state_changes",
        gates: Object.freeze(["visual_response_ms", "inp_ms"]),
      }),
    ]),
  }),
  Object.freeze({
    id: "basalt",
    order: 3,
    risk: "stress",
    article_slug: "theme:basalt",
    accepted_source: "translations/jp/en/pages/theme:basalt/source.wikidot.txt",
    minimum_shape: Object.freeze({bytes: 45000, logical_lines: 1700, css_modules: 7, code_blocks: 6, executable_includes: 16, local_resource_references: 18}),
    required_markers: Object.freeze(["component:interwiki-style", "basalt_scp_logo-for_darkmode.svg", "[[tabview]]"]),
    dependencies: Object.freeze({
      components: Object.freeze(["component:interwiki-style", "component:betterfootnotes", "component:acs-animation", "component:image-block"]),
      assets: Object.freeze([
        "basalt-theme-logo.svg",
        "basalt_scp_logo-for_lightmode.svg",
        "basalt_scp_logo-for_darkmode.svg",
        "RAISA_LIGHTLOGO.png",
        "RAISA_DARKLOGO.png",
        "CLASSIFICATION_LIGHTLOGO.png",
        "CLASSIFICATION_DARKLOGO.png",
        "ETTRA_LIGHTLOGO.png",
        "ETTRA_DARKLOGO.png",
        "ETHICS_LIGHTLOGO.png",
        "ETHICS_DARKLOGO.png",
        "DELTA_T_LIGHTLOGO.png",
        "DELTA_T_DARKLOGO.png",
        "O5_LIGHTLOGO.png",
        "O5_DARKLOGO.png",
        "MISCOMM_LIGHTLOGO.png",
        "MISCOMM_DARKLOGO.png",
      ]),
      remote_local_code: Object.freeze(["theme:basalt/1", "theme:basalt/2", "theme:basalt/3", "theme:basalt/4", "theme:basalt/5", "theme:basalt/6"]),
    }),
    computed_style_probes: Object.freeze([
      ...COMMON_PROBES,
      Object.freeze({id: "basalt_logo", selector: "#header h1 a span", pseudo: "::before", expectation: "required"}),
      Object.freeze({id: "document", selector: ".document, .darkdocument", expectation: "required"}),
      Object.freeze({id: "memo", selector: "div[class$='_memo']", expectation: "required"}),
      Object.freeze({id: "memo_logo", selector: "div[class$='_memo']", pseudo: "::before", expectation: "required"}),
      Object.freeze({id: "tabview", selector: ".yui-navset", expectation: "required"}),
      Object.freeze({id: "style_frame", selector: "iframe[src*='styleFrame.html']", expectation: "optional"}),
    ]),
    interactions: Object.freeze([
      Object.freeze({
        id: "collapsible_toggle",
        action: "click",
        target_selectors: Object.freeze(["details.collapsible-block > summary", ".collapsible-block-folded a"]),
        postcondition: "expanded_state_changes",
        gates: Object.freeze(["visual_response_ms", "inp_ms"]),
      }),
      Object.freeze({
        id: "tab_switch",
        action: "click",
        target_selectors: Object.freeze([".yui-navset .yui-nav li:nth-child(2) a"]),
        postcondition: "selected_tab_changes",
        gates: Object.freeze(["visual_response_ms", "inp_ms"]),
      }),
    ]),
  }),
]);

const ARTIFACT_PATTERNS = Object.freeze([
  Object.freeze({id: "thread_id", pattern: /\bthread[_-]?id\s*[:=]\s*["']?[0-9a-f-]{16,}/giu}),
  Object.freeze({id: "run_id", pattern: /\brun[_-]?id\s*[:=]\s*["']?[a-z0-9-]{8,}/giu}),
  Object.freeze({id: "session_id", pattern: /\b(?:codex[_ -]?)?session[_-]?id\s*[:=]\s*["']?[a-z0-9-]{8,}/giu}),
  Object.freeze({id: "local_absolute_path", pattern: /(?<![A-Za-z0-9._~+@:%-])\/(?:home|Users|mnt|tmp)\/[A-Za-z0-9._~+/@:%-]+/gu}),
  Object.freeze({id: "codex_workspace", pattern: /\b(?:codex-thread-workspaces|codex[_-]?worker|worker[_-]?id\s*[:=])/giu}),
]);

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function countMatches(source, pattern) {
  return [...source.matchAll(pattern)].length;
}

function logicalLineCount(source) {
  if (source.length === 0) return 0;
  const count = source.split(/\r?\n/u).length;
  return source.endsWith("\n") ? count - 1 : count;
}

export function inventoryThemeSource(source) {
  return {
    bytes: Buffer.byteLength(source),
    logical_lines: logicalLineCount(source),
    css_modules: countMatches(source, /\[\[module\s+CSS\]\]/giu),
    code_blocks: countMatches(source, /\[\[code\b/giu),
    executable_includes: countMatches(source, /\[\[include\s+[^\]\s]/giu),
    local_resource_references: countMatches(source, /(?:local--files|local--code)[^\s"'<>)}\]]+/giu),
    css_content_declarations: countMatches(source, /\bcontent\s*:/giu),
  };
}

function lineAndColumn(source, index) {
  const before = source.slice(0, index);
  const lines = before.split(/\r?\n/u);
  return {line: lines.length, column: lines.at(-1).length + 1};
}

export function findSourceArtifactLeaks(source, {limit = 50} = {}) {
  const findings = [];
  for (const {id, pattern} of ARTIFACT_PATTERNS) {
    for (const match of source.matchAll(new RegExp(pattern.source, pattern.flags))) {
      findings.push({id, ...lineAndColumn(source, match.index)});
      if (findings.length >= limit) return findings.sort(compareLeakLocations);
    }
  }
  return findings.sort(compareLeakLocations);
}

function compareLeakLocations(left, right) {
  return left.line - right.line || left.column - right.column || left.id.localeCompare(right.id);
}

export function validateRunId(runId) {
  if (typeof runId !== "string" || !/^[a-z0-9](?:[a-z0-9-]{1,38}[a-z0-9])$/u.test(runId)) {
    throw new Error("--run-id must be 3-40 lowercase ASCII letters, digits, or hyphens, with no edge hyphen");
  }
  return runId;
}

export function validateSiteSlug(siteSlug) {
  if (siteSlug !== ALLOWED_SITE_SLUG) {
    throw new Error(`site is not allowlisted: expected ${ALLOWED_SITE_SLUG}`);
  }
  return siteSlug;
}

export function validateTargetOrigin(value, target) {
  if (target !== "wikidot" && target !== "wikijump") throw new Error(`unknown target: ${target}`);
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error(`${target} origin is not a valid URL`);
  }
  const expected = target === "wikidot"
    ? {protocol: "http:", hostname: `${ALLOWED_SITE_SLUG}.wikidot.com`, ports: new Set([""])}
    : {protocol: "https:", hostname: `${ALLOWED_SITE_SLUG}.wikijump.localhost`, ports: new Set(["", "18443"])};
  if (url.protocol !== expected.protocol || url.hostname !== expected.hostname || !expected.ports.has(url.port) || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
    throw new Error(`${target} origin is outside the hard allowlist`);
  }
  return url.origin;
}

export function runOwnedSlug(runId, tierId) {
  validateRunId(runId);
  if (!THEME_LOCALIZATION_TIERS.some((tier) => tier.id === tierId)) {
    throw new Error(`unknown theme tier: ${tierId}`);
  }
  const slug = `${RUN_OWNED_SLUG_PREFIX}${runId}-${tierId}`;
  assertRunOwnedSlug(slug, runId, tierId);
  return slug;
}

export function assertRunOwnedSlug(slug, runId, tierId) {
  if (!THEME_LOCALIZATION_TIERS.some((tier) => tier.id === tierId)) throw new Error(`unknown theme tier: ${tierId}`);
  const expected = `${RUN_OWNED_SLUG_PREFIX}${validateRunId(runId)}-${tierId}`;
  if (slug !== expected || slug.length > 100 || !/^codex-l10n:[a-z0-9-]+$/u.test(slug)) {
    throw new Error(`page slug is not owned by run ${runId}: ${slug}`);
  }
  return slug;
}

export function assertLegacyRunOwnedSlug(slug, runId, tierId) {
  if (!THEME_LOCALIZATION_TIERS.some((tier) => tier.id === tierId)) throw new Error(`unknown theme tier: ${tierId}`);
  const expected = `${LEGACY_RUN_OWNED_SLUG_PREFIX}${validateRunId(runId)}-${tierId}`;
  if (slug !== expected || slug.length > 100 || !/^theme:codex-l10n-[a-z0-9-]+$/u.test(slug)) {
    throw new Error(`legacy page slug is not owned by run ${runId}: ${slug}`);
  }
  return slug;
}

function isKnownRunOwnedSlug(slug, legacy) {
  if (typeof slug !== "string" || slug.length > 100) return false;
  const prefix = legacy ? LEGACY_RUN_OWNED_SLUG_PREFIX : RUN_OWNED_SLUG_PREFIX;
  for (const tier of THEME_LOCALIZATION_TIERS) {
    const suffix = `-${tier.id}`;
    if (!slug.startsWith(prefix) || !slug.endsWith(suffix)) continue;
    const runId = slug.slice(prefix.length, -suffix.length);
    try {
      (legacy ? assertLegacyRunOwnedSlug : assertRunOwnedSlug)(slug, runId, tier.id);
      return true;
    } catch {
      return false;
    }
  }
  return false;
}

export function isCurrentRunOwnedSlug(slug) {
  return isKnownRunOwnedSlug(slug, false);
}

export function isRecoverableRunOwnedSlug(slug) {
  return isKnownRunOwnedSlug(slug, false) || isKnownRunOwnedSlug(slug, true);
}

export function selectThemeTiers(requested = ["all"]) {
  const selections = requested.length === 0 ? ["all"] : requested;
  const unknown = selections.filter((id) => id !== "all" && !THEME_LOCALIZATION_TIERS.some((tier) => tier.id === id));
  if (unknown.length) throw new Error(`unknown theme tier: ${unknown.join(", ")}`);
  if (selections.includes("all")) return [...THEME_LOCALIZATION_TIERS];
  const selected = new Set(selections);
  return THEME_LOCALIZATION_TIERS.filter((tier) => selected.has(tier.id));
}

async function acceptedSourcePreflight(translationRoot, tier) {
  const sourcePath = path.resolve(translationRoot, tier.accepted_source);
  const relative = path.relative(translationRoot, sourcePath);
  const checks = [];
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`accepted source escapes translation root: ${tier.accepted_source}`);
  }

  let source = "";
  let stat = null;
  try {
    stat = await fs.lstat(sourcePath);
    source = await fs.readFile(sourcePath, "utf8");
  } catch (error) {
    checks.push({id: "accepted_source_readable", status: "fail", detail: error.code ?? error.message});
  }

  if (stat) {
    checks.push({id: "accepted_source_regular_file", status: stat.isFile() && !stat.isSymbolicLink() ? "pass" : "fail"});
  }

  const shape = inventoryThemeSource(source);
  if (stat) {
    checks.push({id: "accepted_source_nonempty", status: source.trim() ? "pass" : "fail"});
    checks.push({id: "accepted_source_contains_japanese", status: /[\u3040-\u30ff\u3400-\u9fff]/u.test(source) ? "pass" : "fail"});
    for (const [field, minimum] of Object.entries(tier.minimum_shape)) {
      checks.push({id: `minimum_${field}`, status: shape[field] >= minimum ? "pass" : "fail", actual: shape[field], expected_minimum: minimum});
    }
    for (const marker of tier.required_markers) {
      checks.push({id: `required_marker:${marker}`, status: source.includes(marker) ? "pass" : "fail"});
    }
    const artifactLeaks = findSourceArtifactLeaks(source);
    checks.push({id: "artifact_leakage", status: artifactLeaks.length === 0 ? "pass" : "fail", findings: artifactLeaks});
  }

  const dependencyFiles = {components: [], assets: []};
  for (const component of tier.dependencies.components) {
    const relativePath = path.join("corpus", "jp", "pages", component, "source.wikidot.txt");
    const result = await inspectDependencyFile(translationRoot, relativePath, component);
    dependencyFiles.components.push(result);
    checks.push({id: `dependency_component:${component}`, status: result.status});
  }
  for (const asset of tier.dependencies.assets) {
    const relativePath = path.join("corpus", "en", "pages", tier.article_slug, "files", asset);
    const result = await inspectDependencyFile(translationRoot, relativePath, asset);
    dependencyFiles.assets.push(result);
    checks.push({id: `dependency_asset:${asset}`, status: result.status});
  }

  const failed = checks.filter((check) => check.status === "fail");
  return {
    status: failed.length === 0 ? "pass" : "fail",
    source: {
      relative_path: tier.accepted_source,
      absolute_path: sourcePath,
      sha256: source ? sha256(source) : null,
      shape,
    },
    dependency_files: dependencyFiles,
    checks,
  };
}

async function inspectDependencyFile(translationRoot, relativePath, name) {
  const filePath = path.resolve(translationRoot, relativePath);
  const relative = path.relative(translationRoot, filePath);
  if (relative.startsWith("..") || path.isAbsolute(relative)) throw new Error(`dependency escapes translation root: ${relativePath}`);
  try {
    const stat = await fs.lstat(filePath);
    if (!stat.isFile() || stat.isSymbolicLink()) return {name, relative_path: relativePath, status: "fail", sha256: null, bytes: 0};
    const contents = await fs.readFile(filePath);
    return {name, relative_path: relativePath, status: "pass", sha256: sha256(contents), bytes: contents.byteLength};
  } catch {
    return {name, relative_path: relativePath, status: "fail", sha256: null, bytes: 0};
  }
}

function buildCleanupContract(resources) {
  return {
    schema: "wikijump_local_lab.run_owned_page_cleanup.v1",
    finally_required: true,
    creation_ledger_required: true,
    delete_only_if_created_by_this_run: true,
    attempt_all_in_reverse_creation_order: true,
    verify_absent_after_delete: true,
    cleanup_on: ["success", "failure", "SIGINT", "SIGTERM"],
    residual_page_policy: "fail_closed",
    resources: [...resources].reverse().map((resource) => ({
      resource_id: resource.resource_id,
      target: resource.target,
      slug: resource.slug,
      action: "delete_page",
      preexisting_policy: "abort_before_write",
      verification: "page_absent",
    })),
  };
}

function pageUrl(origin, slug) {
  return new URL(`/${slug}`, origin).href;
}

export async function buildThemeLocalizationE2EPlan({
  translationRoot,
  runId,
  mode = "dry-run",
  siteSlug = ALLOWED_SITE_SLUG,
  wikidotOrigin = DEFAULT_WIKIDOT_ORIGIN,
  wikijumpOrigin = DEFAULT_WIKIJUMP_ORIGIN,
  tiers = ["all"],
} = {}) {
  if (!translationRoot) throw new Error("translationRoot is required");
  if (!new Set(["dry-run", "execute"]).has(mode)) throw new Error("theme plan mode must be dry-run or execute");
  const validatedRunId = validateRunId(runId);
  const validatedSite = validateSiteSlug(siteSlug);
  const validatedWikidotOrigin = validateTargetOrigin(wikidotOrigin, "wikidot");
  const validatedWikijumpOrigin = validateTargetOrigin(wikijumpOrigin, "wikijump");
  const selectedTiers = selectThemeTiers(tiers);
  const resolvedTranslationRoot = path.resolve(translationRoot);
  const plans = [];
  const resources = [];

  for (const tier of selectedTiers) {
    const slug = runOwnedSlug(validatedRunId, tier.id);
    const preflight = await acceptedSourcePreflight(resolvedTranslationRoot, tier);
    const targets = [
      {id: "wikidot", role: "reference", origin: validatedWikidotOrigin},
      {id: "wikijump", role: "candidate", origin: validatedWikijumpOrigin},
    ].map((target) => {
      const resource = {resource_id: `${tier.id}:${target.id}`, target: target.id, slug};
      resources.push(resource);
      return {...target, url: pageUrl(target.origin, slug), resource_id: resource.resource_id};
    });
    const computedStyles = {properties: themeComputedStyleProperties({properties: COMMON_STYLE_PROPERTIES, probes: tier.computed_style_probes}), probes: tier.computed_style_probes};
    validateThemeComputedStyleContract(computedStyles, {label: `${tier.id} computed-style contract`});
    plans.push({
      id: tier.id,
      order: tier.order,
      risk: tier.risk,
      article_slug: tier.article_slug,
      run_owned_slug: slug,
      preflight,
      run_owned_tags: tier.run_owned_tags ?? [],
      dependencies: tier.dependencies,
      targets,
      capture: {
        viewports: THEME_CAPTURE_VIEWPORTS,
        computed_styles: computedStyles,
        web_vitals: {gates: THEME_PERFORMANCE_GATES, navigation: "cold_context", settle_policy: "observer_buffered_through_lcp_budget"},
        interactions: tier.interactions,
        raw_syntax: {
          detector: "install/local/wikidot-verification/src/render-health.mjs#findRawSyntaxLeaks",
          ignore_rendered_elements: ["script", "style", "pre", "code", ".wj-raw", "span[style*='white-space: pre-wrap']"],
          ignore_source_regions: ["@@...@@", "[[code]]...[[/code]]"],
        },
        artifacts: ["dom.html", "screenshot.png", "computed-styles.json", "web-vitals.json", "performance-attribution.json", "interactions.json", "network-errors.json"],
      },
    });
  }

  const failedTiers = plans.filter((tier) => tier.preflight.status === "fail").map((tier) => tier.id);
  return {
    schema: THEME_LOCALIZATION_E2E_SCHEMA,
    mode,
    run: {id: validatedRunId, site_slug: validatedSite, owned_slug_prefix: `${RUN_OWNED_SLUG_PREFIX}${validatedRunId}-`},
    safety: {
      page_mutations_performed: 0,
      hard_allowlist: {
        site_slug: ALLOWED_SITE_SLUG,
        wikidot_hostname: `${ALLOWED_SITE_SLUG}.wikidot.com`,
        wikijump_hostname: `${ALLOWED_SITE_SLUG}.wikijump.localhost`,
      },
      mirror_sites_are_forbidden: true,
      execute_supported: mode === "execute",
    },
    translation_root: resolvedTranslationRoot,
    preflight: {status: failedTiers.length === 0 ? "pass" : "fail", selected_tiers: plans.length, failed_tiers: failedTiers},
    tiers: plans,
    cleanup: buildCleanupContract(resources),
    reuse: {
      local_page_adapter: "install/local/wikidot-verification/scripts/preview-source.mjs",
      browser_capture: "install/local/wikidot-verification/scripts/capture-browser-rendering.mjs",
      layout_diagnostics: "install/local/wikidot-verification/scripts/layout-diagnostics.mjs",
      latency_capture: "install/local/wikidot-verification/scripts/measure-page-latency.mjs",
      raw_syntax_detector: "install/local/wikidot-verification/src/render-health.mjs#findRawSyntaxLeaks",
    },
    guarded_execution_requirements: [
      "Wikidot authenticated create/edit/delete adapter with a creation ledger",
      "Wikijump adapter allowlist and preexisting-page fail-closed guard",
      "target dependency existence checks and guarded component, attachment, and local-code materialization",
      "finally cleanup executor which verifies both run-owned pages are absent",
      "capture executor for pseudo-element styles, web vitals, and interaction gates",
    ],
  };
}
