import assert from 'node:assert/strict';
import test from 'node:test';

import {
  DEFAULT_IMPORT_USER_ID,
  assertExistingAttachmentMatches,
  attachmentActorUserId,
  deepwellBlobHashHex,
  validateAttachmentActorArgs,
} from '../src/corpus-attachment-policy.mjs';

test('attachment materialization requires an authenticated actor id when using a session token', () => {
  const rows = [{ fullname: 'scp-173', attachments: [{ filename: 'pixel.png' }] }];

  assert.throws(
    () => validateAttachmentActorArgs({ sessionToken: 'token', userId: DEFAULT_IMPORT_USER_ID, attachmentUserId: null }, rows),
    /attachment-user-id|user-id/,
  );
  assert.doesNotThrow(() => validateAttachmentActorArgs({ sessionToken: 'token', userId: 123, attachmentUserId: null }, rows));
  assert.doesNotThrow(() => validateAttachmentActorArgs({ sessionToken: 'token', userId: DEFAULT_IMPORT_USER_ID, attachmentUserId: 123 }, rows));
  assert.doesNotThrow(() => validateAttachmentActorArgs({ sessionToken: 'token', userId: DEFAULT_IMPORT_USER_ID, attachmentUserId: DEFAULT_IMPORT_USER_ID }, rows));
});

test('attachment actor id can differ from page import user id', () => {
  assert.equal(attachmentActorUserId({ userId: DEFAULT_IMPORT_USER_ID, attachmentUserId: 123 }), 123);
  assert.equal(attachmentActorUserId({ userId: 123, attachmentUserId: DEFAULT_IMPORT_USER_ID }), DEFAULT_IMPORT_USER_ID);
  assert.equal(attachmentActorUserId({ userId: 456, attachmentUserId: null }), 456);
  assert.throws(
    () => attachmentActorUserId({ userId: DEFAULT_IMPORT_USER_ID, attachmentUserId: null }, { fullname: 'scp-173' }),
    /scp-173: attachment materialization requires/,
  );
});

test('existing attachments must match the corpus bytes before being skipped', () => {
  const row = { fullname: 'scp-173' };
  const attachment = { filename: 'pixel.png', size: 4 };
  const bytes = Buffer.from([1, 2, 3, 4]);
  const existing = {
    size: bytes.length,
    s3_hash: deepwellBlobHashHex(bytes),
  };

  assert.doesNotThrow(() => assertExistingAttachmentMatches({ row, attachment, existing, bytes }));
  assert.throws(
    () => assertExistingAttachmentMatches({ row, attachment, existing: { ...existing, size: 5 }, bytes }),
    /size mismatch/,
  );
  assert.throws(
    () => assertExistingAttachmentMatches({ row, attachment, existing: { ...existing, s3_hash: '0'.repeat(128) }, bytes }),
    /blob hash mismatch/,
  );
});
