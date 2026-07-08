import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { attachmentS3KeyHex, planDirectAttachmentMaterialization } from '../src/corpus-attachment-direct.mjs';

function sha256Hex(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function writeAttachment(root, filename, bytes) {
  const filePath = path.join(root, filename);
  fs.writeFileSync(filePath, bytes);
  return {
    filename,
    file_path: filePath,
    sha256: sha256Hex(bytes),
    size: bytes.length,
  };
}

test('planDirectAttachmentMaterialization validates bytes and deduplicates by S3 key', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-attachment-direct-'));
  const sharedBytes = Buffer.from([1, 2, 3]);
  const uniqueBytes = Buffer.from([4, 5, 6, 7]);
  const rows = [
    {
      fullname: 'scp-173',
      attachments: [
        writeAttachment(root, 'shared-a.png', sharedBytes),
        writeAttachment(root, 'unique.png', uniqueBytes),
      ],
    },
    {
      fullname: 'scp-174',
      attachments: [
        writeAttachment(root, 'shared-b.png', sharedBytes),
      ],
    },
  ];

  const plan = planDirectAttachmentMaterialization(rows);

  assert.deepEqual(plan.attachment_direct_plan, { attachments_requested: 3, unique_blobs: 2, duplicate_blobs: 1, total_bytes: 10, unique_bytes: 7 });
  assert.equal(plan.blobs.length, 2);
  assert.equal(plan.attachments[0].s3_key_hex, attachmentS3KeyHex(sharedBytes));
  assert.equal(plan.attachments[2].s3_key_hex, plan.attachments[0].s3_key_hex);
  assert.equal(plan.attachments[2].duplicate, true);
  assert.equal(plan.blobs[0].first_file_path, rows[0].attachments[0].file_path);
  assert.equal(plan.blobs[0].mime, null);
});

test('planDirectAttachmentMaterialization rejects stale sha256 metadata', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-attachment-direct-'));
  const attachment = writeAttachment(root, 'pixel.png', Buffer.from([1]));
  attachment.sha256 = '0'.repeat(64);

  assert.throws(
    () => planDirectAttachmentMaterialization([{ fullname: 'scp-173', attachments: [attachment] }]),
    /attachment sha256 mismatch/,
  );
});

test('planDirectAttachmentMaterialization treats missing attachment arrays as empty', () => {
  const plan = planDirectAttachmentMaterialization([{ fullname: 'scp-173' }]);

  assert.deepEqual(plan.attachment_direct_plan, { attachments_requested: 0, unique_blobs: 0, duplicate_blobs: 0, total_bytes: 0, unique_bytes: 0 });
  assert.deepEqual(plan.attachments, []);
  assert.deepEqual(plan.blobs, []);
});
