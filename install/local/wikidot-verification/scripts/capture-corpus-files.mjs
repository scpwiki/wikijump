#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';

import { stableStringify } from '../src/canonical-json.mjs';
import { buildFixtureResourceTargetPath } from '../src/resource-manifest.mjs';
import {
  createHttpFixtureResourceLoader,
  createLocalFixtureResourceLoader,
} from '../src/resource-materializer.mjs';
import { scanForFixtureLocalResources } from '../src/resource-scanner.mjs';

const DEFAULT_BRANCH = 'en';
const DEFAULT_MAX_RESOURCE_BYTES = 64 * 1024 * 1024;
const DEFAULT_HTTP_TIMEOUT_MS = 30_000;
const ATTACHMENT_ONLY_CAPTURE_METHOD = 'wikijump_corpus_attachment_placeholder';
const ATTACHMENT_ONLY_TIMESTAMP = '1970-01-01T00:00:00+00:00';

const MIME_BY_EXTENSION = new Map([
  ['avif', 'image/avif'],
  ['bmp', 'image/bmp'],
  ['css', 'text/css'],
  ['gif', 'image/gif'],
  ['ico', 'image/x-icon'],
  ['jpeg', 'image/jpeg'],
  ['jpg', 'image/jpeg'],
  ['js', 'text/javascript'],
  ['mjs', 'text/javascript'],
  ['mp3', 'audio/mpeg'],
  ['mp4', 'video/mp4'],
  ['otf', 'font/otf'],
  ['png', 'image/png'],
  ['svg', 'image/svg+xml'],
  ['ttf', 'font/ttf'],
  ['webm', 'video/webm'],
  ['webp', 'image/webp'],
  ['woff', 'font/woff'],
  ['woff2', 'font/woff2'],
]);
const RELATIVE_FILE_RE = new RegExp(`\\.(${[...MIME_BY_EXTENSION.keys()].join('|')})(?:$|[?#])`, 'iu');

function parseArgs(argv) {
  const args = {
    corpusRoot: null,
    branch: DEFAULT_BRANCH,
    slug: [],
    slugFile: null,
    dryRun: false,
    createMissingAttachmentPages: false,
    resourceSourceRoot: null,
    maxResourceBytes: DEFAULT_MAX_RESOURCE_BYTES,
    httpTimeoutMs: DEFAULT_HTTP_TIMEOUT_MS,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[index];
    };
    if (arg === '--corpus-root' || arg === '--corpus') args.corpusRoot = next();
    else if (arg === '--branch') args.branch = next();
    else if (arg === '--slug') args.slug.push(next());
    else if (arg === '--slug-file') args.slugFile = next();
    else if (arg === '--dry-run') args.dryRun = true;
    else if (arg === '--create-missing-attachment-pages') args.createMissingAttachmentPages = true;
    else if (arg === '--resource-source-root') args.resourceSourceRoot = next();
    else if (arg === '--max-resource-bytes') args.maxResourceBytes = Number.parseInt(next(), 10);
    else if (arg === '--http-timeout-ms') args.httpTimeoutMs = Number.parseInt(next(), 10);
    else if (arg === '--help' || arg === '-h') {
      console.log(`Usage: capture-corpus-files.mjs --corpus-root <scp-wiki-translation/corpus> [--branch en] [--slug <slug>...] [--slug-file <file>] [--dry-run]

Scans imported Wikidot source pages for Wikidot /local--files/ URLs and captures the referenced bytes into per-page corpus attachment fixtures:

  <corpus-root>/<branch>/pages/<attachment-page>/files/<filename>
  <corpus-root>/<branch>/pages/<attachment-page>/files.json

The files.json manifest is consumed by build-corpus-import-manifest.mjs and then materialized into a local Deepwell runtime by apply-corpus-import-manifest.mjs.

Use --create-missing-attachment-pages only for corpus-adjacent/source-bundle capture roots where cross-page /local--files/<owner>/<file> references need placeholder owner pages for local file serving. It does not vendor files into wikijump source.`);
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!args.corpusRoot) throw new Error('--corpus-root is required');
  if (!Number.isSafeInteger(args.maxResourceBytes) || args.maxResourceBytes <= 0) {
    throw new Error('--max-resource-bytes must be a positive safe integer');
  }
  if (!Number.isSafeInteger(args.httpTimeoutMs) || args.httpTimeoutMs <= 0) {
    throw new Error('--http-timeout-ms must be a positive safe integer');
  }
  return args;
}

async function readSlugSet(args, pagesRoot) {
  const slugs = new Set(args.slug);
  if (args.slugFile !== null) {
    const text = await fs.readFile(args.slugFile, 'utf8');
    for (const line of text.split(/\r?\n/u)) {
      const slug = line.trim();
      if (slug && !slug.startsWith('#')) slugs.add(slug);
    }
  }
  if (slugs.size > 0) return [...slugs].sort();

  const entries = await fs.readdir(pagesRoot, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
}

function sha256Hex(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function deterministicEntityId(branch, page) {
  const bytes = crypto
    .createHash('sha256')
    .update(`wikijump-corpus-attachment-page:${branch}:${page}`)
    .digest()
    .subarray(0, 16);
  bytes[6] = (bytes[6] & 0x0f) | 0x50;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = bytes.toString('hex');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function decodedSegment(segment, field, sourceUrl) {
  try {
    const decoded = decodeURIComponent(segment);
    if (
      decoded.length === 0 ||
      decoded === '.' ||
      decoded === '..' ||
      decoded.includes('/') ||
      decoded.includes('\\') ||
      decoded.includes('\0')
    ) {
      throw new Error('unsafe decoded segment');
    }
    return decoded;
  } catch (error) {
    throw new Error(`${sourceUrl}: invalid ${field} segment ${segment}`, { cause: error });
  }
}

function attachmentPageAndFilename(entry) {
  const parts = entry.wikidot_path.split('/').filter(Boolean);
  if (parts.length < 3 || parts[0] !== 'local--files') {
    throw new Error(`${entry.original_url}: expected /local--files/<page>/<filename>`);
  }
  return {
    page: decodedSegment(parts[1], 'page', entry.original_url),
    filename: decodedSegment(parts.at(-1), 'filename', entry.original_url),
  };
}

function guessMime(filename) {
  const extension = filename.includes('.')
    ? filename.slice(filename.lastIndexOf('.') + 1).toLowerCase()
    : '';
  return MIME_BY_EXTENSION.get(extension) ?? 'application/octet-stream';
}

function kindGuess(filename) {
  const mime = guessMime(filename);
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('audio/')) return 'audio';
  if (mime.startsWith('video/')) return 'video';
  if (mime.startsWith('font/')) return 'font';
  if (mime === 'text/css') return 'css';
  if (mime === 'text/javascript') return 'script';
  return 'unknown';
}

function encodeWikidotSegment(value) {
  return encodeURIComponent(value).replaceAll('%3A', ':');
}

function cleanRelativeFilename(value) {
  let candidate = value.trim().replace(/^['"]|['"]$/gu, '');
  candidate = candidate.split(/[|\s]/u)[0] ?? '';
  candidate = candidate.replace(/[?#].*$/u, '');
  if (
    candidate.length === 0 ||
    /^[a-z][a-z0-9+.-]*:/iu.test(candidate) ||
    candidate.startsWith('/') ||
    candidate.startsWith('#') ||
    candidate.includes('/') ||
    candidate.includes('\\') ||
    candidate.includes('\0') ||
    !RELATIVE_FILE_RE.test(candidate)
  ) {
    return null;
  }
  decodedSegment(encodeURIComponent(candidate), 'filename', candidate);
  return candidate;
}

function relativeAttachmentEntries({ sourceText, slug, sourcePath }) {
  const filenames = new Set();
  const imageDirective = /\[\[[^\]\r\n]*?\bimage\s+([^\]\s|]+)/giu;
  const imageBlockName = /(?:^|[|\s])name\s*=\s*([^\]|\s]+)/giu;
  const cssUrl = /url\(\s*(['"]?)(?![a-z][a-z0-9+.-]*:|\/|#|data:)([^'")\s]+)\1\s*\)/giu;
  const licenseFilename = /Filename:\*\*\s*([^\r\n]+)/giu;

  for (const pattern of [imageDirective, imageBlockName]) {
    for (const match of sourceText.matchAll(pattern)) {
      const filename = cleanRelativeFilename(match[1] ?? '');
      if (filename !== null) filenames.add(filename);
    }
  }
  for (const match of sourceText.matchAll(cssUrl)) {
    const filename = cleanRelativeFilename(match[2] ?? '');
    if (filename !== null) filenames.add(filename);
  }
  for (const match of sourceText.matchAll(licenseFilename)) {
    for (const value of String(match[1] ?? '').split(',')) {
      const filename = cleanRelativeFilename(value);
      if (filename !== null) filenames.add(filename);
    }
  }

  return [...filenames].sort().map((filename) => {
    const site = 'scp-wiki.wikidot.com';
    const wikidotPath = `/local--files/${encodeWikidotSegment(slug)}/${encodeWikidotSegment(filename)}`;
    const originalUrl = `https://${site}${wikidotPath}`;
    return {
      fixture_slug: slug,
      source_path: sourcePath,
      original_url: originalUrl,
      site,
      wikidot_path: wikidotPath,
      filename,
      kind_guess: kindGuess(filename),
      sha256: null,
      local_target_path: buildFixtureResourceTargetPath({
        fixtureSlug: slug,
        site,
        wikidotPath,
      }),
    };
  });
}

function mergeEntries(entries) {
  const byPath = new Map();
  for (const entry of entries) {
    const existing = byPath.get(entry.wikidot_path);
    if (
      existing === undefined ||
      (existing.original_url.startsWith('http://') && entry.original_url.startsWith('https://'))
    ) {
      byPath.set(entry.wikidot_path, entry);
    }
  }
  return [...byPath.values()].sort((left, right) =>
    `${left.wikidot_path}\u0000${left.original_url}`.localeCompare(`${right.wikidot_path}\u0000${right.original_url}`),
  );
}

function toPosixPath(filePath) {
  return filePath.split(path.sep).join(path.posix.sep);
}

async function readExistingManifest(manifestPath) {
  try {
    const text = await fs.readFile(manifestPath, 'utf8');
    const parsed = JSON.parse(text);
    if (!Array.isArray(parsed)) throw new Error('manifest is not an array');
    return parsed;
  } catch (error) {
    if (error?.code === 'ENOENT') return [];
    throw error;
  }
}

async function ensureAttachmentPage({ branch, entry, page, pageDir }) {
  const sourceFilePath = path.join(pageDir, 'source.wikidot.txt');
  try {
    const stat = await fs.stat(sourceFilePath);
    if (!stat.isFile()) throw new Error('not a file');
    return false;
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
  }

  await fs.mkdir(pageDir, { recursive: true });
  const source = [
    'Attachment-only placeholder generated for local Wikijump file materialization.',
    `Original file URL: ${entry.original_url}`,
    '',
  ].join('\n');
  const meta = {
    children: 0,
    commented_at: null,
    commented_by: null,
    comments: 0,
    created_at: ATTACHMENT_ONLY_TIMESTAMP,
    created_by: null,
    fullname: page,
    parent_fullname: null,
    parent_title: null,
    rating: 0,
    revisions: 0,
    tags: ['attachment-placeholder'],
    title: page,
    title_shown: page,
    updated_at: ATTACHMENT_ONLY_TIMESTAMP,
    updated_by: null,
    capture_method: ATTACHMENT_ONLY_CAPTURE_METHOD,
    source_browser_visibility: 'source_only',
    source_visibility_reason: 'attachment_only_placeholder_for_local_file_materialization',
  };

  await fs.writeFile(sourceFilePath, source);
  await fs.writeFile(path.join(pageDir, 'meta.json'), `${stableStringify(meta)}\n`);
  await fs.writeFile(path.join(pageDir, 'entity_id.txt'), `${deterministicEntityId(branch, page)}\n`);
  return true;
}

async function writeAttachment({ corpusRoot, pagesRoot, branch, entry, loadResource, createMissingAttachmentPages }) {
  const { page, filename } = attachmentPageAndFilename(entry);
  const pageDir = path.join(pagesRoot, page);
  const sourceFilePath = path.join(pageDir, 'source.wikidot.txt');
  let createdMissingCorpusPage = false;
  try {
    const stat = await fs.stat(sourceFilePath);
    if (!stat.isFile()) throw new Error('not a file');
  } catch (error) {
    if (error?.code === 'ENOENT') {
      if (createMissingAttachmentPages) {
        createdMissingCorpusPage = await ensureAttachmentPage({ branch, entry, page, pageDir });
      } else {
        return { action: 'skipped_missing_corpus_page', page, filename, original_url: entry.original_url };
      }
    } else {
      throw error;
    }
  }

  const bytes = await loadResource(entry);
  const attachment = {
    filename,
    original_url: entry.original_url,
    wikidot_path: entry.wikidot_path,
    path: `files/${filename}`,
    sha256: sha256Hex(bytes),
    mime: guessMime(filename),
    size: bytes.byteLength,
  };
  const filesDir = path.join(pageDir, 'files');
  const filePath = path.join(filesDir, filename);
  const manifestPath = path.join(pageDir, 'files.json');
  const existing = await readExistingManifest(manifestPath);
  const byPath = new Map(existing.map((item) => [item.wikidot_path, item]));
  byPath.set(attachment.wikidot_path, attachment);
  const merged = [...byPath.values()].sort((left, right) =>
    `${left.wikidot_path}\u0000${left.filename}`.localeCompare(`${right.wikidot_path}\u0000${right.filename}`),
  );

  await fs.mkdir(filesDir, { recursive: true });
  await fs.writeFile(filePath, bytes);
  await fs.writeFile(manifestPath, `${stableStringify(merged)}\n`);
  return {
    action: 'captured',
    page,
    filename,
    bytes: bytes.byteLength,
    sha256: attachment.sha256,
    corpus_path: toPosixPath(path.relative(corpusRoot, filePath)),
    original_url: entry.original_url,
    created_missing_corpus_page: createdMissingCorpusPage,
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const corpusRoot = path.resolve(args.corpusRoot);
  const pagesRoot = path.join(corpusRoot, args.branch, 'pages');
  const slugs = await readSlugSet(args, pagesRoot);
  const loadResource = args.resourceSourceRoot === null
    ? createHttpFixtureResourceLoader({
      maxResourceBytes: args.maxResourceBytes,
      timeoutMs: args.httpTimeoutMs,
      redirect: 'follow',
    })
    : createLocalFixtureResourceLoader({
      sourceRoot: path.resolve(args.resourceSourceRoot),
      maxResourceBytes: args.maxResourceBytes,
    });

  const summary = {
    dry_run: args.dryRun,
    branch: args.branch,
    scanned_pages: 0,
    discovered_resources: 0,
    captured: 0,
    skipped_missing_corpus_page: 0,
    created_missing_corpus_page: 0,
    out_of_scope: 0,
  };

  for (const slug of slugs) {
    const sourcePath = path.join(pagesRoot, slug, 'source.wikidot.txt');
    let sourceText;
    try {
      sourceText = await fs.readFile(sourcePath, 'utf8');
    } catch (error) {
      if (error?.code === 'ENOENT') {
        console.error(JSON.stringify({ slug, action: 'skipped_missing_source' }));
        continue;
      }
      throw error;
    }

    summary.scanned_pages += 1;
    const sourcePathForManifest = toPosixPath(path.relative(corpusRoot, sourcePath));
    const scanned = scanForFixtureLocalResources({
      sourceText,
      fixtureSlug: slug,
      sourcePath: sourcePathForManifest,
    });
    const entries = mergeEntries([
      ...scanned.manifest,
      ...relativeAttachmentEntries({ sourceText, slug, sourcePath: sourcePathForManifest }),
    ]);
    summary.out_of_scope += scanned.out_of_scope.length;
    summary.discovered_resources += entries.length;

    for (const entry of entries) {
      const { page, filename } = attachmentPageAndFilename(entry);
      if (args.dryRun) {
        const pageSourcePath = path.join(pagesRoot, page, 'source.wikidot.txt');
        let missingCorpusPage = false;
        try {
          const stat = await fs.stat(pageSourcePath);
          missingCorpusPage = !stat.isFile();
        } catch (error) {
          if (error?.code !== 'ENOENT') throw error;
          missingCorpusPage = true;
        }
        const action = missingCorpusPage && args.createMissingAttachmentPages
          ? 'would_create_missing_corpus_page'
          : 'would_capture';
        console.log(JSON.stringify({ action, source_slug: slug, page, filename, original_url: entry.original_url }));
        continue;
      }
      const result = await writeAttachment({
        corpusRoot,
        pagesRoot,
        branch: args.branch,
        entry,
        loadResource,
        createMissingAttachmentPages: args.createMissingAttachmentPages,
      });
      summary[result.action] = (summary[result.action] ?? 0) + 1;
      if (result.created_missing_corpus_page) summary.created_missing_corpus_page += 1;
      console.log(JSON.stringify({ source_slug: slug, ...result }));
    }
  }

  console.log(JSON.stringify({ summary }, null, 2));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
