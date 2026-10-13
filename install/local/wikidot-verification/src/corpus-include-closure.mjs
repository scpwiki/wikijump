// Shared Wikidot include-dependency closure for corpus import tooling.
//
// A mirrored page renders through every page it includes (themes,
// components, info blocks), so importing a page without its transitive
// include dependencies leaves "does not exist" error blocks in otherwise
// healthy articles (Refs #2093). This module walks that closure over corpus
// sources so the manifest builder can materialize it. The same-site include
// contract matches the drift reporter (scripts/report-mirror-include-drift.mjs):
// includes inside @@...@@ spans and [!-- --] comments are display-only
// documentation, variable-dependent targets ({{...}}) cannot be resolved
// statically, and cross-site targets name another site's page.

export const INCLUDE_PATTERN = /\[\[\s*include\s+([^\s\]|]+)/giu;
const SLUG_PATTERN = /^[a-z0-9][a-z0-9:_-]*$/u;
const CROSS_SITE_PATTERN = /^:([^:]+):(.+)$/u;
const VARIABLE_PATTERN = /\{\$/u;

export const DEFAULT_CLOSURE_MAX_DEPTH = 32;
export const DEFAULT_CLOSURE_MAX_PAGES = 10000;

/**
 * Normalize an include target to a local slug, or null when it names another
 * site, depends on an include variable, or is not a page name (for example
 * escaped documentation text).
 */
export function includeTargetSlug(target, sourceSite) {
  let slug = target.trim();
  if (slug.includes('{$')) return null;
  const crossSite = CROSS_SITE_PATTERN.exec(slug);
  if (crossSite) {
    if (crossSite[1].toLowerCase() !== sourceSite) return null;
    slug = crossSite[2];
  }
  slug = slug.toLowerCase();
  return SLUG_PATTERN.test(slug) ? slug : null;
}

/**
 * Remove display-only spans from a source text. Raw @@...@@ spans and
 * [!-- --] comments can document include syntax without invoking it;
 * includes inside code blocks remain active under Wikidot's include
 * contract.
 */
export function stripInactiveSourceSpans(sourceText) {
  return sourceText
    .replace(/@@[^\r\n]*?@@/gu, '')
    .replace(/\[!--[\s\S]*?--\]/gu, '');
}

/**
 * Classify every include target in a source text.
 *
 * Returns `{ sameSite, external, dynamic }`: resolvable same-site slugs in
 * first-seen order, cross-site `site:page` identities, and the count of
 * variable-dependent targets that cannot be resolved statically.
 */
export function parseIncludeTargets(sourceText, sourceSite) {
  const sameSite = [];
  const seenSameSite = new Set();
  const external = [];
  const seenExternal = new Set();
  let dynamic = 0;
  const activeSource = stripInactiveSourceSpans(sourceText);
  // The pattern is global; reset position so repeated calls are deterministic.
  INCLUDE_PATTERN.lastIndex = 0;
  for (const match of activeSource.matchAll(INCLUDE_PATTERN)) {
    const raw = match[1].trim();
    if (VARIABLE_PATTERN.test(raw)) {
      dynamic += 1;
      continue;
    }
    const crossSite = CROSS_SITE_PATTERN.exec(raw);
    if (crossSite && crossSite[1].toLowerCase() !== sourceSite.toLowerCase()) {
      const identity = `${crossSite[1].toLowerCase()}:${crossSite[2].toLowerCase()}`;
      if (!seenExternal.has(identity)) {
        seenExternal.add(identity);
        external.push(identity);
      }
      continue;
    }
    const slug = includeTargetSlug(raw, sourceSite);
    if (slug !== null && !seenSameSite.has(slug)) {
      seenSameSite.add(slug);
      sameSite.push(slug);
    }
  }
  INCLUDE_PATTERN.lastIndex = 0;
  return { sameSite, external, dynamic };
}

/**
 * Breadth-first transitive include closure over same-site corpus sources.
 *
 * `readSource(slug)` returns the source text or null when the corpus has no
 * such page; `pageExists(slug)` reports corpus membership. Roots come first
 * at depth 0, each slug is visited once, and cycles terminate through the
 * seen set. Traversal stops expanding past `maxDepth` (deeper targets are
 * reported as `truncated`) and throws past `maxPages` total visited slugs.
 *
 * Returns `{ included, unresolved, external, dynamic, truncated }` where
 * `included` holds `{ slug, depth, via }` in visit order, `unresolved` holds
 * same-site targets absent from the corpus, and `external` holds cross-site
 * identities that this single-site expansion cannot materialize.
 */
export function expandIncludeClosure({
  roots,
  readSource,
  pageExists,
  sourceSite,
  maxDepth = DEFAULT_CLOSURE_MAX_DEPTH,
  maxPages = DEFAULT_CLOSURE_MAX_PAGES,
}) {
  if (!Array.isArray(roots) || roots.length === 0) {
    throw new Error('include closure requires at least one root slug');
  }
  if (!Number.isInteger(maxDepth) || maxDepth < 0) {
    throw new Error('include closure maxDepth must be a non-negative integer');
  }
  if (!Number.isInteger(maxPages) || maxPages <= 0) {
    throw new Error('include closure maxPages must be a positive integer');
  }
  const included = [];
  const seen = new Set();
  const unresolved = [];
  const seenUnresolved = new Set();
  const external = [];
  const seenExternal = new Set();
  let dynamic = 0;
  const truncated = [];
  const seenTruncated = new Set();
  const queue = roots.map((slug) => ({ slug, depth: 0, via: null }));
  while (queue.length > 0) {
    const { slug, depth, via } = queue.shift();
    if (seen.has(slug)) continue;
    seen.add(slug);
    if (seen.size > maxPages) {
      throw new Error(`include closure exceeds maxPages ${maxPages} at ${slug}`);
    }
    const source = readSource(slug);
    included.push({ slug, depth, via });
    if (source === null) {
      if (depth > 0 && !seenUnresolved.has(slug)) {
        seenUnresolved.add(slug);
        unresolved.push(slug);
      }
      continue;
    }
    if (depth >= maxDepth) {
      const targets = parseIncludeTargets(source, sourceSite);
      for (const target of targets.sameSite) {
        if (!seen.has(target) && !seenTruncated.has(target)) {
          seenTruncated.add(target);
          truncated.push(target);
        }
      }
      dynamic += targets.dynamic;
      for (const identity of targets.external) {
        if (!seenExternal.has(identity)) {
          seenExternal.add(identity);
          external.push(identity);
        }
      }
      continue;
    }
    const targets = parseIncludeTargets(source, sourceSite);
    dynamic += targets.dynamic;
    for (const identity of targets.external) {
      if (!seenExternal.has(identity)) {
        seenExternal.add(identity);
        external.push(identity);
      }
    }
    for (const target of targets.sameSite) {
      if (seen.has(target)) continue;
      if (!pageExists(target)) {
        if (!seenUnresolved.has(target)) {
          seenUnresolved.add(target);
          unresolved.push(target);
        }
        continue;
      }
      queue.push({ slug: target, depth: depth + 1, via: slug });
    }
  }
  return { included, unresolved, external, dynamic, truncated };
}
