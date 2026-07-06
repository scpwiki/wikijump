// Dependency-closure resolution for the local Wikidot rendering lab (P2).
//
// Discovers page dependencies (includes, theme/component pages, CSS modules,
// parent pages, /local--files/ attachments) from wikitext and metadata,
// resolves them against the registered source bundles, and reports a
// fail-closed verdict per page. Out-of-bundle dependencies are recorded and
// classified — never silently substituted, dropped, or widened (LP-1 rule
// applied to closure; see dependency-closure.spec.md).

const INCLUDE_REGEX = /\[\[include\s+(:?)([^\s\]|]+)/gi;
const CSS_MODULE_REGEX = /\[\[module\s+css\b/gi;
const DATA_MODULE_REGEX = /\[\[module\s+(listpages|countpages|backlinks)\b/gi;
const LOCAL_FILE_REGEX = /\/local--files\/(?<page>[^/\s"'\]]+)\/(?<file>[^\s"'\])?]+)/gi;

// Regions whose contents are display-only and must not contribute
// dependencies: @@raw@@ spans and [[code]] blocks.
const ESCAPED_REGION_PATTERNS = [
  // Wikidot @@raw@@ spans are inline-only; matching across newlines would
  // let a stray @@ shift pairing for the rest of the document.
  /@@.*?@@/g,
  /\[\[code[^\]]*\]\][\s\S]*?\[\[\/code\]\]/gi,
  // Wikidot comment blocks, which frequently carry usage examples.
  /\[!--[\s\S]*?--\]/g,
];

export function stripEscapedRegions(wikitext) {
  let stripped = wikitext;
  for (const pattern of ESCAPED_REGION_PATTERNS) {
    stripped = stripped.replace(pattern, '');
  }
  return stripped;
}

// Parse one include target. Wikidot cross-site includes lead with a colon
// (`[[include :scp-wiki:component:foo]]`); everything else is same-site,
// where colons are category separators (`[[include component:foo]]`).
export function parseIncludeTarget(rawTarget, leadingColon) {
  const target = rawTarget.replace(/\|.*$/, '').trim();
  if (leadingColon) {
    const separator = target.indexOf(':');
    if (separator > 0) {
      return { site: target.slice(0, separator), page: target.slice(separator + 1) };
    }
  }
  return { site: null, page: target };
}

export function isThemeOrComponentSlug(slug) {
  return /^(theme|component):/.test(slug);
}

export function scanWikitextDependencies(wikitext) {
  const stripped = stripEscapedRegions(wikitext ?? '');
  const includes = [];
  for (const match of stripped.matchAll(INCLUDE_REGEX)) {
    const { site, page } = parseIncludeTarget(match[2], match[1] === ':');
    includes.push({ site, page, raw: match[0] });
  }
  const localFiles = [];
  for (const match of stripped.matchAll(LOCAL_FILE_REGEX)) {
    localFiles.push({ page: match.groups.page, file: match.groups.file });
  }
  return {
    includes,
    cssModuleCount: (stripped.match(CSS_MODULE_REGEX) ?? []).length,
    dataModules: [...stripped.matchAll(DATA_MODULE_REGEX)].map((m) => m[1].toLowerCase()),
    localFiles,
  };
}

// Families whose bundle natively owns a local site. Non-native rows can
// declare the same local_site (e.g. JP rows materialized under scp-wiki for
// cross-site includes); on key collisions the native row must win so EN
// pages resolve against EN sources.
const NATIVE_SITE_FAMILY = new Map([
  ['scp-wiki', 'EN'],
  ['scp-jp', 'JP'],
]);

function isNativeRow(row, site) {
  const nativeFamily = NATIVE_SITE_FAMILY.get(site);
  return nativeFamily === undefined || row.family === undefined || row.family === nativeFamily;
}

// Registry over the registered source bundles: key `${site}:${slug}`.
// Returns the map; collisions between rows claiming the same key are
// recorded on `registry.collisions` (never silently resolved by insertion
// order alone).
export function buildBundleRegistry(rows) {
  const registry = new Map();
  const collisions = [];
  for (const row of rows) {
    const site = row.local_site ?? row.source_site ?? null;
    const slug = row.slug ?? row.fullname ?? null;
    if (site === null || slug === null) continue;
    const key = `${site}:${slug}`;
    const existing = registry.get(key);
    if (existing === undefined) {
      registry.set(key, row);
      continue;
    }
    collisions.push({
      key,
      kept: dependencyLabel(isNativeRow(existing, site) ? existing : row),
      dropped: dependencyLabel(isNativeRow(existing, site) ? row : existing),
    });
    if (!isNativeRow(existing, site) && isNativeRow(row, site)) {
      registry.set(key, row);
    }
  }
  registry.collisions = collisions;
  return registry;
}

function dependencyLabel(row) {
  return row.fixture_id ?? `${row.local_site ?? row.source_site}:${row.slug ?? row.fullname}`;
}

// Resolve the closure for one target row. Walks includes transitively
// (bounded by maxDepth), collecting an import order of
// parents -> theme/component -> other includes -> target.
export function resolveDependencyClosure({ row, registry, readSource, maxDepth = 8 }) {
  const site = row.local_site ?? row.source_site;
  const inBundle = [];
  const outOfBundle = [];
  const cycles = [];
  const missingFiles = [];
  const dataModules = new Set();
  const seen = new Set();
  const activePath = [];

  function visit(depSite, depSlug, kind, depth) {
    const key = `${depSite}:${depSlug}`;
    if (activePath.includes(key)) {
      cycles.push([...activePath.slice(activePath.indexOf(key)), key]);
      return;
    }
    if (seen.has(key)) return;
    seen.add(key);
    const depRow = registry.get(key);
    if (!depRow) {
      outOfBundle.push({ dependency: key, kind });
      return;
    }
    if (depth >= maxDepth) {
      outOfBundle.push({ dependency: key, kind: 'max-depth-exceeded' });
      return;
    }
    activePath.push(key);
    expand(depRow, depth + 1);
    activePath.pop();
    inBundle.push({ label: dependencyLabel(depRow), kind, site: depSite, slug: depSlug });
  }

  function expand(pageRow, depth) {
    const pageSite = pageRow.local_site ?? pageRow.source_site;
    const parent = pageRow.parent_fullname ?? null;
    if (parent) visit(pageSite, parent, 'parent', depth);
    const source = readSource(pageRow);
    if (source === null) {
      outOfBundle.push({ dependency: dependencyLabel(pageRow), kind: 'source-unreadable' });
      return;
    }
    const scan = scanWikitextDependencies(source);
    for (const include of scan.includes) {
      const includeSite = include.site ?? pageSite;
      const kind = isThemeOrComponentSlug(include.page) ? 'theme-component' : 'include';
      visit(includeSite, include.page, kind, depth);
    }
    for (const moduleName of scan.dataModules) dataModules.add(moduleName);
    for (const ref of scan.localFiles) {
      missingFiles.push({ page: ref.page, file: ref.file, site: pageSite });
    }
  }

  const targetKey = `${site}:${row.slug ?? row.fullname}`;
  seen.add(targetKey);
  activePath.push(targetKey);
  expand(row, 0);
  activePath.pop();

  // Import order: parents first, then theme/component, then other includes.
  const kindRank = { parent: 0, 'theme-component': 1, include: 2 };
  const importOrder = [...inBundle]
    .sort((a, b) => (kindRank[a.kind] ?? 3) - (kindRank[b.kind] ?? 3))
    .map((dep) => dep.label);
  importOrder.push(dependencyLabel(row));

  // A page textually including itself (usage examples, ListUsers-style
  // pagination tricks) renders fine on live Wikidot under its include-depth
  // cap; classify separately from genuine multi-page cycles.
  const selfIncludesOnly =
    cycles.length > 0 && cycles.every((cycle) => new Set(cycle).size === 1);
  let status = 'closure_complete';
  if (cycles.length > 0) status = selfIncludesOnly ? 'self_include_cycle' : 'cycle';
  else if (outOfBundle.length > 0) status = 'out_of_bundle';

  return {
    fixture_id: dependencyLabel(row),
    dependencies: {
      in_bundle: inBundle.map((dep) => dep.label),
      out_of_bundle: outOfBundle,
      cycles,
      missing_files: missingFiles,
    },
    data_modules: [...dataModules],
    import_order: importOrder,
    status,
  };
}

const CLASSIFIED_OUT_OF_BUNDLE_KINDS = new Set([
  'include',
  'theme-component',
  'parent',
  'max-depth-exceeded',
  'source-unreadable',
]);

export function summarizeClosureReports(reports) {
  const statusCounts = {};
  let unclassified = 0;
  for (const report of reports) {
    statusCounts[report.status] = (statusCounts[report.status] ?? 0) + 1;
    for (const dep of report.dependencies.out_of_bundle) {
      if (!CLASSIFIED_OUT_OF_BUNDLE_KINDS.has(dep.kind)) unclassified += 1;
    }
  }
  return {
    page_count: reports.length,
    status_counts: statusCounts,
    unclassified_out_of_bundle: unclassified,
    exit_code: unclassified > 0 ? 1 : 0,
  };
}
