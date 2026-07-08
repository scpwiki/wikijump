import assert from 'node:assert/strict';
import test from 'node:test';

import { buildAttachmentStagingSql, parseAttachmentStagingResults } from '../src/corpus-attachment-staging-sql.mjs';

const S3_HASH = 'a'.repeat(128);

function sampleSql(overrides = {}) {
  return buildAttachmentStagingSql({
    siteId: 246,
    actorUserId: 135,
    revisionComments: 'attachment import',
    attachments: [{ fullname: 'scp-173', filename: "statue's file.png", sha256: 'b'.repeat(64), size: 123, s3_key_hex: S3_HASH, mime: 'image/png' }],
    ...overrides,
  });
}

function assertSqlFragments(sql, fragments) {
  for (const fragment of fragments) assert.ok(sql.includes(fragment), `missing SQL fragment:\n${fragment}\n\nSQL:\n${sql}`);
}

test('buildAttachmentStagingSql stages planned attachments and guard joins', () => {
  const sql = sampleSql();

  assertSqlFragments(sql, [
    'WITH planned_attachments AS',
    "(0::integer, 'scp-173'::text, 'statue''s file.png'::text",
    "decode('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hex')",
    'p.site_id = 246::bigint',
    'p.slug = pa.fullname',
    'p.deleted_at IS NULL',
    'planned_name_counts AS',
    'GROUP BY page_id, filename',
    'f.page_id = pm.page_id',
    'f.name = pm.filename',
    'f.deleted_at IS NULL',
    'bb.s3_hash = pm.s3_hash',
  ]);
});

test('buildAttachmentStagingSql classifies rows and exposes first revision staging fields', () => {
  const sql = sampleSql();

  assertSqlFragments(sql, [
    "WHEN af.active_file_count = 1 AND lfr.size = pm.size AND lfr.s3_hash = pm.s3_hash THEN 'skip_existing'",
    "WHEN af.active_file_count = 1 THEN 'existing_mismatch'",
    "WHEN bb.s3_hash IS NOT NULL THEN 'blob_blacklisted'",
    "WHEN pm.page_id IS NULL THEN 'missing_page'",
    "WHEN COALESCE(pnc.planned_name_count, 1) > 1 THEN 'duplicate_planned_name'",
    "ELSE 'insert'",
    'staged_file_rows AS',
    'staged_first_revisions AS',
    "'create'::text AS revision_type",
    '0::integer AS revision_number',
    '135::bigint AS user_id',
    "ARRAY['page', 'name', 'blob', 'mime']::text[] AS changes",
    'ARRAY[]::text[] AS hidden',
  ]);
  assert.equal(/\bINSERT\s+INTO\b/iu.test(sql), false);
});

test('buildAttachmentStagingSql commit mode inserts files and first revisions', () => {
  const sql = sampleSql({ commit: true });

  assertSqlFragments(sql, [
    'inserted_files AS',
    'INSERT INTO file (site_id, page_id, name, from_wikidot)',
    'inserted_file_rows AS',
    'inserted_first_revisions AS',
    'INSERT INTO file_revision (',
    'JOIN inserted_file_rows ifr',
    'COALESCE(inserted_file_rows.file_id::text, c.file_id::text, \'\') AS file_id',
    'COALESCE(inserted_first_revisions.revision_number::text, \'\') AS revision_number',
  ]);
});

test('buildAttachmentStagingSql handles empty input and validates metadata', () => {
  assertSqlFragments(sampleSql({ attachments: [] }), ['WHERE false', 'NULL::integer AS row_index', 'ORDER BY c.row_index;']);
  assert.throws(() => sampleSql({ attachments: [{ fullname: 'scp-173', filename: 'bad.bin', sha256: 'bad', size: 1, s3_key_hex: S3_HASH }] }), /sha256/);
  assert.throws(() => sampleSql({ attachments: [{ fullname: 'scp-173', filename: 'bad.bin', sha256: 'b'.repeat(64), size: 1, s3_key_hex: 'bad' }] }), /s3_key_hex/);
});

test('parseAttachmentStagingResults summarizes pipe-delimited rows', () => {
  const parsed = parseAttachmentStagingResults([
    '0|scp-173|a.png|insert||101||0',
    '1|scp-173|a.png|skip_existing||101|201|',
    '2|missing|b.txt|fail_closed|missing_page|||',
    '3|scp-174|blocked.gif|fail_closed|blob_blacklisted|102||',
    '4|scp-175|old.bin|fail_closed|existing_mismatch|103|203|',
  ].join('\n'));

  assert.deepEqual(parsed.summary, { total: 5, insert: 1, skip_existing: 1, fail_closed: 3 });
  assert.deepEqual(parsed.rows[0], { row_index: 0, fullname: 'scp-173', filename: 'a.png', action: 'insert', reason: null, page_id: 101, file_id: null, revision_number: 0 });
  assert.equal(parsed.rows[4].reason, 'existing_mismatch');
  assert.throws(() => parseAttachmentStagingResults('0|scp-173|a.png|defer||||'), /unknown action/);
  assert.throws(() => parseAttachmentStagingResults('0|too|few'), /expected 8 pipe-delimited fields/);
});
