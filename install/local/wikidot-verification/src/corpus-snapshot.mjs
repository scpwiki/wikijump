import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { stableStringify } from './canonical-json.mjs';

const CANONICAL_BRANCH_PATHS = new Set(['index.json', 'by-uuid', 'pages', 'posts']);
const WIKIDOT_SITE_SLUG_RE = /[a-z0-9](?:[a-z0-9-]*[a-z0-9])?/u;
const WIKIDOT_TARGET_RE = /(?:https?:\/\/)?([a-z0-9](?:[a-z0-9-]*[a-z0-9])?)\.wikidot\.com\/?/u;

function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function sha256(bufferOrString) {
  return crypto.createHash('sha256').update(bufferOrString).digest('hex');
}

function unixPath(value) {
  return value.split(path.sep).join('/');
}

function hashFile(filePath) {
  const bytes = fs.readFileSync(filePath);
  return { bytes: bytes.length, sha256: sha256(bytes) };
}

function walkFiles(root, relativeRoot = '') {
  const output = [];
  const stack = [{ absolute: root, relative: relativeRoot }];
  while (stack.length > 0) {
    const current = stack.pop();
    const entries = fs.readdirSync(current.absolute, { withFileTypes: true })
      .sort((left, right) => codePointCompare(right.name, left.name));
    for (const entry of entries) {
      const absolute = path.join(current.absolute, entry.name);
      const relative = current.relative ? `${current.relative}/${entry.name}` : entry.name;
      if (entry.isDirectory()) stack.push({ absolute, relative });
      else if (entry.isFile()) output.push({ absolute, relative });
    }
  }
  return output.sort((left, right) => codePointCompare(left.relative, right.relative));
}

function canonicalWikidotSiteSlug(value) {
  const match = typeof value === 'string' ? WIKIDOT_SITE_SLUG_RE.exec(value) : null;
  return typeof value === 'string'
    && value.length <= 63
    && match?.[0] === value
    ? value
    : null;
}

function siteSlugFromTargetWiki(value) {
  const directSlug = canonicalWikidotSiteSlug(value);
  if (directSlug !== null) return directSlug;
  if (typeof value !== 'string') return null;
  const match = WIKIDOT_TARGET_RE.exec(value);
  return match === null || match[0] !== value ? null : canonicalWikidotSiteSlug(match[1]);
}

function sitePageUrl(site, hostnameSuffix, fullname) {
  const url = new URL(`https://${site}.${hostnameSuffix}/`);
  url.pathname = `/${fullname}`;
  return url.href;
}

function sourceSiteFromIndex(index, branch) {
  const sites = new Set();
  let hasUnverifiedCandidate = false;
  const addCandidate = (value, normalizer = canonicalWikidotSiteSlug) => {
    const site = normalizer(value);
    if (site === null) hasUnverifiedCandidate = true;
    else sites.add(site);
  };

  if (index?.by_site_created_at !== undefined) {
    if (index.by_site_created_at === null
      || typeof index.by_site_created_at !== 'object'
      || Array.isArray(index.by_site_created_at)) {
      hasUnverifiedCandidate = true;
    } else {
      for (const key of Object.keys(index.by_site_created_at)) {
        const separator = key.indexOf('|');
        if (separator <= 0) hasUnverifiedCandidate = true;
        else addCandidate(key.slice(0, separator));
      }
    }
  }
  if (index?.target_wiki !== undefined) {
    addCandidate(index.target_wiki, siteSlugFromTargetWiki);
  }
  const siteStatus = hasUnverifiedCandidate
    ? 'unverified'
    : sites.size === 1
      ? 'resolved'
      : sites.size === 0
        ? 'missing'
        : 'ambiguous';
  return {
    source_site: siteStatus === 'resolved' ? [...sites][0] : null,
    source_sites: [...sites].sort(codePointCompare),
    site_status: siteStatus,
    branch,
  };
}

function readJson(filePath) {
  try {
    return { value: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { value: null, error: String(error?.message ?? error) };
  }
}

function fileRecord(corpusRoot, entry, cache) {
  let integrity = cache.get(entry.absolute);
  if (integrity === undefined) {
    integrity = hashFile(entry.absolute);
    cache.set(entry.absolute, integrity);
  }
  return {
    path: unixPath(path.relative(corpusRoot, entry.absolute)),
    bytes: integrity.bytes,
    sha256: integrity.sha256,
  };
}

function treeSha256(files) {
  return sha256(files.map((file) => `${file.path}\0${file.bytes}\0${file.sha256}\n`).join(''));
}

function pageRow({
  branch,
  branchRoot,
  corpusRoot,
  directoryName,
  site,
  fileByPath,
  pageFiles,
}) {
  const pageRoot = path.join(branchRoot, 'pages', directoryName);
  const relativePageRoot = unixPath(path.relative(corpusRoot, pageRoot));
  const sourcePath = `${relativePageRoot}/source.wikidot.txt`;
  const metaPath = `${relativePageRoot}/meta.json`;
  const entityPath = `${relativePageRoot}/entity_id.txt`;
  const metaRead = fs.existsSync(path.join(pageRoot, 'meta.json'))
    ? readJson(path.join(pageRoot, 'meta.json'))
    : { value: null, error: 'missing meta.json' };
  const fullname = typeof metaRead.value?.fullname === 'string' ? metaRead.value.fullname : directoryName;
  const problems = [];
  for (const required of [sourcePath, metaPath, entityPath]) {
    if (!fileByPath.has(required)) problems.push(`missing:${path.posix.basename(required)}`);
  }
  if (metaRead.error !== null) problems.push(`meta:${metaRead.error}`);
  if (fullname !== directoryName) problems.push(`fullname_mismatch:${fullname}`);
  if (site.site_status !== 'resolved') problems.push(`source_site_${site.site_status}`);

  const sourceSite = site.source_site;
  return {
    fixture_id: `${branch.toUpperCase()}:${fullname}`,
    family: branch.toUpperCase(),
    slug: fullname,
    fullname,
    source_branch: branch,
    source_site: sourceSite,
    local_site: sourceSite,
    source_url: sourceSite === null ? null : sitePageUrl(sourceSite, 'wikidot.com', fullname),
    local_https_url: sourceSite === null ? null : sitePageUrl(sourceSite, 'wikijump.localhost', fullname),
    source_artifact: fileByPath.has(sourcePath) ? path.join(corpusRoot, sourcePath) : null,
    source_path: fileByPath.has(sourcePath) ? path.join(corpusRoot, sourcePath) : null,
    source_sha256: fileByPath.get(sourcePath)?.sha256 ?? null,
    source_bytes: fileByPath.get(sourcePath)?.bytes ?? null,
    meta_artifact: fileByPath.has(metaPath) ? path.join(corpusRoot, metaPath) : null,
    meta_sha256: fileByPath.get(metaPath)?.sha256 ?? null,
    entity_id_artifact: fileByPath.has(entityPath) ? path.join(corpusRoot, entityPath) : null,
    entity_id_sha256: fileByPath.get(entityPath)?.sha256 ?? null,
    parent_fullname: metaRead.value?.parent_fullname ?? null,
    required_browser: true,
    inventory_status: problems.length === 0 ? 'ready' : 'invalid',
    inventory_problems: problems,
    file_count: pageFiles.length,
    page_tree_sha256: treeSha256(pageFiles),
  };
}

export function discoverCorpusBranches(corpusRoot) {
  return fs.readdirSync(corpusRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && !entry.name.startsWith('_'))
    .filter((entry) => fs.existsSync(path.join(corpusRoot, entry.name, 'pages')))
    .map((entry) => entry.name)
    .sort(codePointCompare);
}

export function discoverCanonicalCorpusFiles(corpusRoot, branches = null) {
  const resolvedCorpusRoot = path.resolve(corpusRoot);
  const selectedBranches = branches ?? discoverCorpusBranches(resolvedCorpusRoot);
  const files = [];
  for (const branch of selectedBranches) {
    const branchRoot = path.join(resolvedCorpusRoot, branch);
    for (const canonicalPath of CANONICAL_BRANCH_PATHS) {
      const absolute = path.join(branchRoot, canonicalPath);
      if (!fs.existsSync(absolute)) continue;
      const stat = fs.statSync(absolute);
      if (stat.isFile()) files.push({ absolute, relative: `${branch}/${canonicalPath}` });
      else {
        for (const file of walkFiles(absolute, `${branch}/${canonicalPath}`)) files.push(file);
      }
    }
  }
  return files.sort((left, right) => codePointCompare(left.relative, right.relative));
}

export function buildCorpusSnapshot({
  corpusRoot,
  branches = null,
  repositories = [],
  fileIntegrityCache = new Map(),
}) {
  const resolvedCorpusRoot = path.resolve(corpusRoot);
  const selectedBranches = branches ?? discoverCorpusBranches(resolvedCorpusRoot);
  const cache = fileIntegrityCache;
  const rows = [];
  const branchRecords = [];

  for (const branch of selectedBranches) {
    const branchRoot = path.join(resolvedCorpusRoot, branch);
    const indexPath = path.join(branchRoot, 'index.json');
    const indexRead = fs.existsSync(indexPath) ? readJson(indexPath) : { value: null, error: 'missing index.json' };
    const site = sourceSiteFromIndex(indexRead.value, branch);
    const allFiles = [];
    for (const canonicalPath of CANONICAL_BRANCH_PATHS) {
      const absolute = path.join(branchRoot, canonicalPath);
      if (!fs.existsSync(absolute)) continue;
      const stat = fs.statSync(absolute);
      if (stat.isFile()) allFiles.push({ absolute, relative: `${branch}/${canonicalPath}` });
      else {
        for (const file of walkFiles(absolute, `${branch}/${canonicalPath}`)) allFiles.push(file);
      }
    }
    const files = allFiles.map((entry) => fileRecord(resolvedCorpusRoot, entry, cache));
    const fileByPath = new Map(files.map((file) => [file.path, file]));
    const pageFilesByDirectory = new Map();
    const pagesPrefix = `${branch}/pages/`;
    for (const file of files) {
      if (!file.path.startsWith(pagesPrefix)) continue;
      const relative = file.path.slice(pagesPrefix.length);
      const separator = relative.indexOf('/');
      if (separator === -1) continue;
      const directoryName = relative.slice(0, separator);
      const pageFiles = pageFilesByDirectory.get(directoryName) ?? [];
      pageFiles.push(file);
      pageFilesByDirectory.set(directoryName, pageFiles);
    }
    const pagesRoot = path.join(branchRoot, 'pages');
    const pageDirectories = fs.readdirSync(pagesRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort(codePointCompare);
    const branchRows = pageDirectories.map((directoryName) => pageRow({
      branch,
      branchRoot,
      corpusRoot: resolvedCorpusRoot,
      directoryName,
      site,
      fileByPath,
      pageFiles: pageFilesByDirectory.get(directoryName) ?? [],
    }));
    rows.push(...branchRows);
    branchRecords.push({
      branch,
      ...site,
      corpus_kind: indexRead.value?.corpus_kind ?? 'wikidot_site_corpus',
      index_error: indexRead.error,
      page_count: branchRows.length,
      ready_page_count: branchRows.filter((row) => row.inventory_status === 'ready').length,
      invalid_page_count: branchRows.filter((row) => row.inventory_status !== 'ready').length,
      file_count: files.length,
      byte_count: files.reduce((sum, file) => sum + file.bytes, 0),
      tree_sha256: treeSha256(files),
      files,
    });
  }

  const stableBody = {
    schema: 'wikijump_full_parity.corpus_inventory_lock.v1',
    corpus_root: resolvedCorpusRoot,
    repositories,
    branches: branchRecords,
    rows,
    totals: {
      branch_count: branchRecords.length,
      page_count: rows.length,
      ready_page_count: rows.filter((row) => row.inventory_status === 'ready').length,
      invalid_page_count: rows.filter((row) => row.inventory_status !== 'ready').length,
      file_count: branchRecords.reduce((sum, branch) => sum + branch.file_count, 0),
      byte_count: branchRecords.reduce((sum, branch) => sum + branch.byte_count, 0),
    },
  };
  return { ...stableBody, manifest_sha256: sha256(stableStringify(stableBody)) };
}
