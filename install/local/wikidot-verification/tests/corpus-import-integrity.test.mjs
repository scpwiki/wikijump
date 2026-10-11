import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  buildIntegrityQuery,
  readIntegrityManifest,
  verifyIntegrityRows,
} from '../scripts/verify-corpus-import-integrity.mjs';

function manifestRow(root, fullname, source) {
  const sourcePath = path.join(root, `${fullname}.wikidot.txt`);
  const bytes = Buffer.from(source, 'utf8');
  fs.writeFileSync(sourcePath, bytes);
  return {
    fullname,
    source_path: sourcePath,
    source_sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
    source_bytes: bytes.length,
  };
}

test('integrity manifest validates source bytes and exact slug selection', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-integrity-'));
  const rows = [manifestRow(root, 'scp-173', 'コンテンツ\n'), manifestRow(root, 'scp-174', 'Second')];
  const text = rows.map((row) => JSON.stringify(row)).join('\n');

  assert.deepEqual(readIntegrityManifest(text, ['scp-174']), [rows[1]]);
  assert.throws(() => readIntegrityManifest(text, ['scp-999']), /does not contain selected page scp-999/);
  fs.writeFileSync(rows[0].source_path, 'changed');
  assert.throws(() => readIntegrityManifest(text, ['scp-173']), /source_path hash mismatch/);
});

test('integrity query safely requests selected pages and retrieves provenance and body length', () => {
  const query = buildIntegrityQuery({siteId: 6000006}, [{fullname: "scp-'173"}]);
  assert.match(query, /page\.site_id = 6000006/);
  assert.match(query, /wikidot_page_snapshot/);
  assert.match(query, /wikidot_corpus_import_item/);
  assert.match(query, /translate\(encode\(convert_to\(source\.contents, 'UTF8'\), 'base64'\), E'\\n', ''\)/);
  assert.match(query, /scp-''173/);
});

test('integrity report passes matching source, provenance, import state, and required body', () => {
  const bytes = Buffer.from('SCP-173\n', 'utf8');
  const sha = crypto.createHash('sha256').update(bytes).digest('hex');
  const rows = [{fullname: 'scp-173', source_sha256: sha}];
  const result = verifyIntegrityRows(rows, `scp-173|30000001|30000002|${bytes.toString('base64')}|${sha}|42|done`, {
    renderedBodySlugs: ['scp-173'],
  });

  assert.equal(result.status, 'pass');
  assert.deepEqual(result.failures, []);
  assert.equal(result.pages[0].current_source_matches_manifest, true);
  assert.equal(result.pages[0].provenance_matches_manifest, true);
  assert.equal(result.pages[0].rendered_body_characters, 42);
});

test('integrity report does not accept a shell-ready page as a completed import', () => {
  const bytes = Buffer.from('SCP-173\n', 'utf8');
  const sha = crypto.createHash('sha256').update(bytes).digest('hex');
  const result = verifyIntegrityRows(
    [{fullname: 'scp-173', source_sha256: sha}],
    `scp-173|30000001|30000002|${bytes.toString('base64')}|${sha}|42|shell_ready`,
    {renderedBodySlugs: ['scp-173']},
  );

  assert.equal(result.status, 'fail');
  assert.deepEqual(result.failures.map((failure) => failure.reason), ['import_not_done:shell_ready']);
});

test('integrity report fails closed for missing pages and source, provenance, import, or body mismatches', () => {
  const bytes = Buffer.from('\n', 'utf8');
  const sha = crypto.createHash('sha256').update('real source').digest('hex');
  const rows = [
    {fullname: 'blank-page', source_sha256: sha},
    {fullname: 'missing-page', source_sha256: sha},
  ];
  const result = verifyIntegrityRows(rows, `blank-page|30000003|30000004|${bytes.toString('base64')}|${'0'.repeat(64)}|0|failed`, {
    renderedBodySlugs: ['blank-page'],
  });

  assert.equal(result.status, 'fail');
  assert.deepEqual(result.failures.map((failure) => failure.reason), [
    'current_source_mismatch',
    'missing_or_mismatched_provenance',
    'import_not_done:failed',
    'rendered_body_empty',
    'missing_page',
  ]);
});

test('integrity report rejects malformed numeric fields instead of treating them as usable evidence', () => {
  const bytes = Buffer.from('source', 'utf8');
  const sha = crypto.createHash('sha256').update(bytes).digest('hex');
  assert.throws(() => verifyIntegrityRows(
    [{fullname: 'scp-173', source_sha256: sha}],
    `scp-173|30000001|30000002|${bytes.toString('base64')}|${sha}|not-a-length|done`,
  ), /invalid numeric fields for scp-173/);
});
