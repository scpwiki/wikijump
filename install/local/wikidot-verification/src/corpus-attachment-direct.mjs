import crypto from 'node:crypto';
import fs from 'node:fs';

const SHA256_RE = /^[0-9a-f]{64}$/u;

function addSafeInteger(left, right, label) {
  const value = left + right;
  if (!Number.isSafeInteger(value)) {
    throw new Error(`${label} exceeds safe integer range`);
  }
  return value;
}

function assertAttachmentShape(row, attachment) {
  if (attachment === null || typeof attachment !== 'object' || Array.isArray(attachment)) {
    throw new Error(`${row.fullname}: attachment entry must be an object`);
  }
  if (typeof attachment.filename !== 'string' || attachment.filename.length === 0) {
    throw new Error(`${row.fullname}: attachment filename must be a non-empty string`);
  }
  if (typeof attachment.file_path !== 'string' || attachment.file_path.length === 0) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment file_path must be a non-empty string`);
  }
  if (typeof attachment.sha256 !== 'string' || !SHA256_RE.test(attachment.sha256)) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment sha256 must be a lowercase sha256 hex string`);
  }
  if (!Number.isSafeInteger(attachment.size) || attachment.size < 0) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment size must be a non-negative safe integer`);
  }
}

function readVerifiedAttachmentBytes(row, attachment) {
  assertAttachmentShape(row, attachment);
  const bytes = fs.readFileSync(attachment.file_path);
  const actualSha256 = crypto.createHash('sha256').update(bytes).digest('hex');
  if (actualSha256 !== attachment.sha256) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment sha256 mismatch: expected ${attachment.sha256}, got ${actualSha256}`);
  }
  if (bytes.length !== attachment.size) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment size mismatch: expected ${attachment.size}, got ${bytes.length}`);
  }
  return bytes;
}

export function attachmentS3KeyHex(bytes) {
  return crypto.createHash('sha512').update(bytes).digest('hex');
}

export function planDirectAttachmentMaterialization(rows) {
  const blobsByS3Key = new Map();
  const attachments = [];
  let totalBytes = 0;
  let uniqueBytes = 0;

  for (const row of rows) {
    const rowAttachments = Array.isArray(row.attachments) ? row.attachments : [];
    for (const attachment of rowAttachments) {
      const bytes = readVerifiedAttachmentBytes(row, attachment);
      const s3KeyHex = attachmentS3KeyHex(bytes);
      totalBytes = addSafeInteger(totalBytes, bytes.length, 'attachment total_bytes');

      let blob = blobsByS3Key.get(s3KeyHex);
      const duplicate = blob !== undefined;
      if (!blob) {
        blob = { s3_key_hex: s3KeyHex, sha256: attachment.sha256, size: bytes.length, first_fullname: row.fullname, first_filename: attachment.filename };
        blobsByS3Key.set(s3KeyHex, blob);
        uniqueBytes = addSafeInteger(uniqueBytes, bytes.length, 'attachment unique_bytes');
      }

      attachments.push({ fullname: row.fullname, filename: attachment.filename, file_path: attachment.file_path, sha256: attachment.sha256, size: bytes.length, s3_key_hex: s3KeyHex, duplicate });
    }
  }

  const attachmentsRequested = attachments.length;
  const uniqueBlobs = blobsByS3Key.size;
  return {
    attachment_direct_plan: {
      attachments_requested: attachmentsRequested,
      unique_blobs: uniqueBlobs,
      duplicate_blobs: attachmentsRequested - uniqueBlobs,
      total_bytes: totalBytes,
      unique_bytes: uniqueBytes,
    },
    attachments,
    blobs: [...blobsByS3Key.values()],
  };
}
