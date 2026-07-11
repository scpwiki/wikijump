import crypto from 'node:crypto';

const SHA512_RE = /^[0-9a-f]{128}$/u;

function assertPositiveSafeInteger(value, name) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${name} must be a positive safe integer`);
  }
}

function assertNonNegativeSafeInteger(value, name) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${name} must be a non-negative safe integer`);
  }
}

function addSafeInteger(left, right, label) {
  const value = left + right;
  if (!Number.isSafeInteger(value)) {
    throw new Error(`${label} exceeds safe integer range`);
  }
  return value;
}

function asBlobBuffer(value) {
  if (Buffer.isBuffer(value)) {
    return value;
  }
  if (value instanceof Uint8Array) {
    return Buffer.from(value.buffer, value.byteOffset, value.byteLength);
  }
  if (value instanceof ArrayBuffer) {
    return Buffer.from(value);
  }
  throw new TypeError('readBlobBytes must return Buffer, Uint8Array, or ArrayBuffer bytes');
}

function sha512Hex(bytes) {
  return crypto.createHash('sha512').update(bytes).digest('hex');
}

function assertBlobShape(blob, index) {
  if (blob === null || typeof blob !== 'object' || Array.isArray(blob)) {
    throw new TypeError(`blobs[${index}] must be an object`);
  }
  if (typeof blob.s3_key_hex !== 'string' || !SHA512_RE.test(blob.s3_key_hex)) {
    throw new TypeError(`blobs[${index}].s3_key_hex must be a lowercase sha512 hex string`);
  }
  assertNonNegativeSafeInteger(blob.size, `blobs[${index}].size`);
  if (blob.mime !== undefined && blob.mime !== null && typeof blob.mime !== 'string') {
    throw new TypeError(`blobs[${index}].mime must be a string when provided`);
  }
}

function validateInputs({blobs, objectStore, readBlobBytes, concurrency}) {
  if (!Array.isArray(blobs)) {
    throw new TypeError('blobs must be an array');
  }
  if (objectStore === null || typeof objectStore !== 'object') {
    throw new TypeError('objectStore must be an object');
  }
  if (typeof objectStore.headObject !== 'function') {
    throw new TypeError('objectStore.headObject must be a function');
  }
  if (typeof objectStore.putObject !== 'function') {
    throw new TypeError('objectStore.putObject must be a function');
  }
  if (typeof readBlobBytes !== 'function') {
    throw new TypeError('readBlobBytes must be a function');
  }
  assertPositiveSafeInteger(concurrency, 'concurrency');

  let totalBytes = 0;
  const keys = new Set();
  for (const [index, blob] of blobs.entries()) {
    assertBlobShape(blob, index);
    if (keys.has(blob.s3_key_hex)) {
      throw new TypeError(`blobs[${index}].s3_key_hex duplicates an earlier blob`);
    }
    keys.add(blob.s3_key_hex);
    totalBytes = addSafeInteger(totalBytes, blob.size, 'attachment upload total_bytes');
  }
  return totalBytes;
}

function assertExistingHead(head, blob, phase) {
  if (head === null || typeof head !== 'object' || Array.isArray(head)) {
    throw new Error(`${phase} HEAD returned an invalid response for ${blob.s3_key_hex}`);
  }
  if (head.exists !== true) {
    throw new Error(`${phase} HEAD did not find uploaded object ${blob.s3_key_hex}`);
  }
  if (!Number.isSafeInteger(head.size) || head.size < 0) {
    throw new Error(`${phase} HEAD returned an invalid size for ${blob.s3_key_hex}`);
  }
  if (head.size !== blob.size) {
    throw new Error(`${phase} HEAD size mismatch for ${blob.s3_key_hex}: expected ${blob.size}, got ${head.size}`);
  }
}

async function uploadOneBlob({blob, objectStore, readBlobBytes}) {
  const initialHead = await objectStore.headObject(blob.s3_key_hex);
  if (initialHead?.exists === true) {
    assertExistingHead(initialHead, blob, 'initial');
    return {key: blob.s3_key_hex, action: 'skipped_existing', bytes: 0};
  }
  if (initialHead !== undefined && initialHead !== null && initialHead?.exists !== false) {
    throw new Error(`initial HEAD returned an invalid response for ${blob.s3_key_hex}`);
  }

  const bytes = asBlobBuffer(await readBlobBytes(blob));
  if (bytes.byteLength !== blob.size) {
    throw new Error(`blob byte length mismatch for ${blob.s3_key_hex}: expected ${blob.size}, got ${bytes.byteLength}`);
  }

  const actualSha512 = sha512Hex(bytes);
  if (actualSha512 !== blob.s3_key_hex) {
    throw new Error(`blob sha512 mismatch for ${blob.s3_key_hex}: got ${actualSha512}`);
  }

  const contentType = blob.mime ?? 'application/octet-stream';
  await objectStore.putObject(blob.s3_key_hex, bytes, {contentType});
  const verificationHead = await objectStore.headObject(blob.s3_key_hex);
  assertExistingHead(verificationHead, blob, 'verification');
  return {key: blob.s3_key_hex, action: 'uploaded', bytes: bytes.byteLength};
}

export async function uploadPlannedAttachmentBlobs({
  blobs,
  objectStore,
  readBlobBytes,
  concurrency = 16,
} = {}) {
  const totalBytes = validateInputs({blobs, objectStore, readBlobBytes, concurrency});
  const results = new Array(blobs.length);
  const workerCount = Math.min(concurrency, blobs.length);
  let nextIndex = 0;

  async function worker() {
    while (nextIndex < blobs.length) {
      const index = nextIndex;
      nextIndex += 1;
      const blob = blobs[index];
      try {
        results[index] = await uploadOneBlob({blob, objectStore, readBlobBytes});
      } catch (error) {
        results[index] = {
          key: blob.s3_key_hex,
          action: 'failed',
          error: error instanceof Error ? error.message : String(error),
        };
      }
    }
  }

  await Promise.all(Array.from({length: workerCount}, () => worker()));

  let uploaded = 0;
  let skippedExisting = 0;
  let failed = 0;
  let uploadedBytes = 0;
  for (const result of results) {
    if (result.action === 'uploaded') {
      uploaded += 1;
      uploadedBytes = addSafeInteger(uploadedBytes, result.bytes, 'attachment upload uploaded_bytes');
    } else if (result.action === 'skipped_existing') {
      skippedExisting += 1;
    } else if (result.action === 'failed') {
      failed += 1;
    }
  }

  return {
    requested: blobs.length,
    uploaded,
    skipped_existing: skippedExisting,
    failed,
    total_bytes: totalBytes,
    uploaded_bytes: uploadedBytes,
    results,
  };
}
