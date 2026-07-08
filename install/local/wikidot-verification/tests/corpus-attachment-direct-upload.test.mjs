import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import test from 'node:test';

import {uploadPlannedAttachmentBlobs} from '../src/corpus-attachment-direct-upload.mjs';
import {createHttpObjectStoreClient} from '../src/corpus-attachment-object-store.mjs';

function sha512Hex(bytes) {
  return crypto.createHash('sha512').update(bytes).digest('hex');
}

function blobFromBytes(bytes, extras = {}) {
  return {s3_key_hex: sha512Hex(bytes), size: bytes.byteLength, ...extras};
}

function createFakeObjectStore(existing = new Map()) {
  const objects = new Map(existing);
  const calls = [];
  return {
    calls,
    async headObject(key) {
      calls.push({method: 'HEAD', key});
      const bytes = objects.get(key);
      if (bytes === undefined) {
        return {exists: false};
      }
      return {exists: true, size: bytes.byteLength};
    },
    async putObject(key, bytes, options) {
      calls.push({method: 'PUT', key, bytes: Buffer.from(bytes), options});
      objects.set(key, Buffer.from(bytes));
    },
  };
}

test('uploadPlannedAttachmentBlobs uploads missing blobs and skips matching existing blobs', async () => {
  const existingBytes = Buffer.from('already present');
  const uploadBytes = Buffer.from('new bytes');
  const existingBlob = blobFromBytes(existingBytes, {mime: 'text/plain'});
  const uploadBlob = blobFromBytes(uploadBytes, {mime: 'image/png'});
  const objectStore = createFakeObjectStore(new Map([[existingBlob.s3_key_hex, existingBytes]]));
  const readKeys = [];

  const summary = await uploadPlannedAttachmentBlobs({
    blobs: [existingBlob, uploadBlob],
    objectStore,
    readBlobBytes: async (blob) => {
      readKeys.push(blob.s3_key_hex);
      return uploadBytes;
    },
    concurrency: 2,
  });

  assert.deepEqual(summary, {
    requested: 2,
    uploaded: 1,
    skipped_existing: 1,
    failed: 0,
    total_bytes: existingBytes.byteLength + uploadBytes.byteLength,
    uploaded_bytes: uploadBytes.byteLength,
    results: [
      {key: existingBlob.s3_key_hex, action: 'skipped_existing', bytes: 0},
      {key: uploadBlob.s3_key_hex, action: 'uploaded', bytes: uploadBytes.byteLength},
    ],
  });
  assert.deepEqual(readKeys, [uploadBlob.s3_key_hex]);
  assert.deepEqual(
    objectStore.calls.map((call) => ({method: call.method, key: call.key, contentType: call.options?.contentType})),
    [
      {method: 'HEAD', key: existingBlob.s3_key_hex, contentType: undefined},
      {method: 'HEAD', key: uploadBlob.s3_key_hex, contentType: undefined},
      {method: 'PUT', key: uploadBlob.s3_key_hex, contentType: 'image/png'},
      {method: 'HEAD', key: uploadBlob.s3_key_hex, contentType: undefined},
    ],
  );
});

test('uploadPlannedAttachmentBlobs uses default content type and verifies sha512 before PUT', async () => {
  const expectedBytes = Buffer.from('expected bytes');
  const badBytes = Buffer.from('wrong bytes');
  const blob = blobFromBytes(expectedBytes);
  const objectStore = createFakeObjectStore();

  const summary = await uploadPlannedAttachmentBlobs({
    blobs: [blob],
    objectStore,
    readBlobBytes: async () => badBytes,
    concurrency: 1,
  });

  assert.equal(summary.uploaded, 0);
  assert.equal(summary.failed, 1);
  assert.match(summary.results[0].error, /byte length mismatch|sha512 mismatch/);
  assert.deepEqual(objectStore.calls, [{method: 'HEAD', key: blob.s3_key_hex}]);

  const secondStore = createFakeObjectStore();
  const secondSummary = await uploadPlannedAttachmentBlobs({
    blobs: [blob],
    objectStore: secondStore,
    readBlobBytes: async () => expectedBytes,
    concurrency: 1,
  });

  assert.equal(secondSummary.uploaded, 1);
  assert.equal(secondStore.calls[1].options.contentType, 'application/octet-stream');
});

test('uploadPlannedAttachmentBlobs collects per-blob failures and continues', async () => {
  const goodBytes = Buffer.from('good');
  const failingBytes = Buffer.from('failing');
  const goodBlob = blobFromBytes(goodBytes);
  const failingBlob = blobFromBytes(failingBytes);
  const objects = new Map();
  const objectStore = {
    async headObject(key) {
      if (key === failingBlob.s3_key_hex) {
        throw new Error('head unavailable');
      }
      const bytes = objects.get(key);
      return bytes === undefined ? {exists: false} : {exists: true, size: bytes.byteLength};
    },
    async putObject(key, bytes) {
      objects.set(key, Buffer.from(bytes));
    },
  };

  const summary = await uploadPlannedAttachmentBlobs({
    blobs: [failingBlob, goodBlob],
    objectStore,
    readBlobBytes: async (blob) => (blob.s3_key_hex === goodBlob.s3_key_hex ? goodBytes : failingBytes),
    concurrency: 2,
  });

  assert.equal(summary.requested, 2);
  assert.equal(summary.uploaded, 1);
  assert.equal(summary.failed, 1);
  assert.equal(summary.results[0].action, 'failed');
  assert.match(summary.results[0].error, /head unavailable/);
  assert.equal(summary.results[1].action, 'uploaded');
});

test('uploadPlannedAttachmentBlobs enforces bounded concurrency', async () => {
  const blobs = Array.from({length: 5}, (_, index) => Buffer.from(`blob-${index}`)).map((bytes) => blobFromBytes(bytes));
  const bytesByKey = new Map(blobs.map((blob, index) => [blob.s3_key_hex, Buffer.from(`blob-${index}`)]));
  let activeHeads = 0;
  let maxActiveHeads = 0;
  const objects = new Map();
  let releaseHead;
  const firstTwoHeadsStarted = new Promise((resolve) => {
    releaseHead = resolve;
  });
  let startedHeads = 0;
  const objectStore = {
    async headObject(key) {
      activeHeads += 1;
      maxActiveHeads = Math.max(maxActiveHeads, activeHeads);
      startedHeads += 1;
      if (startedHeads === 2) {
        releaseHead();
      }
      await firstTwoHeadsStarted;
      activeHeads -= 1;
      const bytes = objects.get(key);
      return bytes === undefined ? {exists: false} : {exists: true, size: bytes.byteLength};
    },
    async putObject(key, bytes) {
      objects.set(key, Buffer.from(bytes));
    },
  };

  const summary = await uploadPlannedAttachmentBlobs({
    blobs,
    objectStore,
    readBlobBytes: async (blob) => bytesByKey.get(blob.s3_key_hex),
    concurrency: 2,
  });

  assert.equal(summary.uploaded, 5);
  assert.equal(maxActiveHeads, 2);
});

test('uploadPlannedAttachmentBlobs rejects invalid inputs before object-store calls', async () => {
  const objectStore = createFakeObjectStore();
  await assert.rejects(
    uploadPlannedAttachmentBlobs({
      blobs: [{s3_key_hex: 'not-hex', size: 1}],
      objectStore,
      readBlobBytes: async () => Buffer.from([]),
    }),
    /s3_key_hex/,
  );
  await assert.rejects(
    uploadPlannedAttachmentBlobs({
      blobs: [],
      objectStore,
      readBlobBytes: async () => Buffer.from([]),
      concurrency: 0,
    }),
    /concurrency/,
  );
  assert.deepEqual(objectStore.calls, []);
});

test('createHttpObjectStoreClient validates configuration and refuses unauthenticated network use', async () => {
  assert.throws(
    () => createHttpObjectStoreClient({endpoint: 'ftp://localhost:9000', bucket: 'wikijump', accessKeyId: 'key', secretAccessKey: 'secret'}),
    /endpoint/,
  );

  const client = createHttpObjectStoreClient({
    endpoint: 'http://localhost:9000',
    bucket: 'wikijump',
    accessKeyId: 'key',
    secretAccessKey: 'secret',
  });
  await assert.rejects(client.headObject('a'.repeat(128)), /SigV4/);
  await assert.rejects(client.putObject('a'.repeat(128), Buffer.from([]), {contentType: 'application/octet-stream'}), /SigV4/);
});
