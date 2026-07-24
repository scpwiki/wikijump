import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  materializeCorpusRowAttachments,
  readCorpusAttachmentBytes,
} from '../src/corpus-import-attachments.mjs';

function attachmentFixture(bytes = Buffer.from('fixture attachment')) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'wikijump-corpus-attachment-'));
  const filePath = path.join(directory, 'fixture.txt');
  fs.writeFileSync(filePath, bytes);
  return {
    cleanup() {
      fs.rmSync(directory, { recursive: true, force: true });
    },
    row: { fullname: 'scp-173' },
    attachment: {
      filename: 'fixture.txt',
      file_path: filePath,
      size: bytes.byteLength,
      sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
    },
  };
}

test('readCorpusAttachmentBytes verifies file bytes against corpus metadata', () => {
  const fixture = attachmentFixture();
  try {
    assert.deepEqual(readCorpusAttachmentBytes(fixture.row, fixture.attachment), Buffer.from('fixture attachment'));
    assert.throws(
      () => readCorpusAttachmentBytes(fixture.row, { ...fixture.attachment, size: 1 }),
      /attachment size mismatch/,
    );
  } finally {
    fixture.cleanup();
  }
});

test('materializeCorpusRowAttachments defers skipped attachments without RPC work', async () => {
  const result = await materializeCorpusRowAttachments({
    args: { skipAttachments: true },
    row: { fullname: 'scp-173', attachments: [{ filename: 'fixture.txt' }] },
    pageId: 173,
    getFile: async () => assert.fail('getFile must not run for deferred attachments'),
    rpc: async () => assert.fail('rpc must not run for deferred attachments'),
  });

  assert.deepEqual(result, {
    attachments_requested: 1,
    attachments_uploaded: 0,
    attachments_skipped_existing: 0,
    attachments_deferred: 1,
  });
});
