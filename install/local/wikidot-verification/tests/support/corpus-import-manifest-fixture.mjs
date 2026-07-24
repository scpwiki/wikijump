import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export function writePage(root, branch, fullname, { entityId, meta = {}, source = 'content' } = {}) {
  const pageDir = path.join(root, branch, 'pages', fullname);
  fs.mkdirSync(pageDir, { recursive: true });
  const completeMeta = {
    children: 0,
    commented_at: null,
    commented_by: null,
    comments: 0,
    created_at: '2008-07-25T20:49:21+00:00',
    created_by: 'Lt Masipag',
    fullname,
    parent_fullname: null,
    parent_title: null,
    rating: 10634,
    revisions: 57,
    tags: ['scp', 'euclid'],
    title: 'SCP-173',
    title_shown: 'SCP-173',
    updated_at: '2025-04-02T12:17:27+00:00',
    updated_by: 'ParallelPotatoes',
    ...meta,
  };
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), source);
  fs.writeFileSync(path.join(pageDir, 'meta.json'), `${JSON.stringify(completeMeta, null, 2)}\n`);
  fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), `${entityId}\n`);
}

export function writePageAttachment(root, branch, fullname, { filename, bytes, originalUrl } = {}) {
  const pageDir = path.join(root, branch, 'pages', fullname);
  const filesDir = path.join(pageDir, 'files');
  const filePath = path.join(filesDir, filename);
  fs.mkdirSync(filesDir, { recursive: true });
  fs.writeFileSync(filePath, bytes);
  fs.writeFileSync(
    path.join(pageDir, 'files.json'),
    `${JSON.stringify([
      {
        filename,
        original_url: originalUrl,
        wikidot_path: new URL(originalUrl).pathname,
        path: `files/${filename}`,
        sha256: cryptoSha256(bytes),
        mime: 'image/png',
        size: bytes.length,
      },
    ], null, 2)}\n`,
  );
}

export function writeSourceBundlePage(root, fullname, { entityId = null, site = 'sandbox-for-codex', meta = {}, manifest = {}, source = 'content' } = {}) {
  const pageDir = path.join(root, 'pages', fullname);
  fs.mkdirSync(pageDir, { recursive: true });
  const sourceBytes = Buffer.byteLength(source);
  const sourceSha256 = cryptoSha256(source);
  const completeMeta = {
    capture_method: 'wikidot_xmlrpc_pages.get_one',
    category: '_default',
    children_count: '0',
    comments_count: 0,
    fullname,
    name: fullname,
    parent_fullname: null,
    rating: 0,
    revisions_count: 1,
    size: sourceBytes,
    source_bytes: sourceBytes,
    source_sha256: sourceSha256,
    tags: ['codex'],
    title: fullname,
    title_shown: fullname,
    votes_count: 0,
    xmlrpc_fullname: fullname,
    ...meta,
  };
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), source);
  fs.writeFileSync(path.join(pageDir, 'meta.json'), `${JSON.stringify(completeMeta, null, 2)}\n`);
  if (entityId !== null) fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), `${entityId}\n`);
  const manifestPath = path.join(root, 'corpus-manifest.tsv');
  const baseHeaders = ['site', 'fullname', 'title', 'tags', 'source_path', 'source_bytes', 'source_sha256', 'meta_path', 'capture_method'];
  const manifestHeaders = [...baseHeaders, ...Object.keys(manifest)];
  if (!fs.existsSync(manifestPath)) {
    fs.writeFileSync(manifestPath, `${manifestHeaders.join('\t')}\n`);
  }
  const baseColumns = {
    site,
    fullname,
    title: completeMeta.title,
    tags: completeMeta.tags.join('|'),
    source_path: path.join(pageDir, 'source.wikidot.txt'),
    source_bytes: String(sourceBytes),
    source_sha256: sourceSha256,
    meta_path: path.join(pageDir, 'meta.json'),
    capture_method: completeMeta.capture_method,
  };
  fs.appendFileSync(manifestPath, `${manifestHeaders.map((header) => baseColumns[header] ?? manifest[header] ?? '').join('\t')}\n`);
}

export function cryptoSha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}
