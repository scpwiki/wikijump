import crypto from 'node:crypto';

export const DEFAULT_IMPORT_USER_ID = -1;

export function rowsHaveAttachments(rows) {
  return rows.some((row) => Array.isArray(row.attachments) && row.attachments.length > 0);
}

export function validateAttachmentActorArgs(args, rows) {
  if (args.skipAttachments) return;
  if (!rowsHaveAttachments(rows)) return;
  if (!args.sessionToken) return;
  if ((args.attachmentUserId ?? null) === null && args.userId === DEFAULT_IMPORT_USER_ID) {
    throw new Error('attachment materialization with an authenticated session requires --attachment-user-id or --user-id matching the session user');
  }
}

export function attachmentActorUserId(args, row = null) {
  const hasExplicitAttachmentUser = (args.attachmentUserId ?? null) !== null;
  const userId = hasExplicitAttachmentUser ? args.attachmentUserId : args.userId;
  if (!Number.isInteger(userId)) {
    throw new Error('--attachment-user-id or --user-id must be an integer');
  }
  if (!hasExplicitAttachmentUser && userId === DEFAULT_IMPORT_USER_ID) {
    const prefix = row?.fullname ? `${row.fullname}: ` : '';
    throw new Error(`${prefix}attachment materialization requires --attachment-user-id or --user-id matching the authenticated session user`);
  }
  return userId;
}

export function deepwellBlobHashHex(bytes) {
  if (bytes.length === 0) return '0'.repeat(128);
  return crypto.createHash('sha512').update(bytes).digest('hex');
}

export function assertExistingAttachmentMatches({ row, attachment, existing, bytes }) {
  if (existing === null) return;
  const label = `${row.fullname}/${attachment.filename}`;
  if (existing.size !== attachment.size) {
    throw new Error(`${label}: existing attachment size mismatch: expected ${attachment.size}, got ${existing.size}`);
  }
  const expectedHash = deepwellBlobHashHex(bytes);
  const actualHash = typeof existing.s3_hash === 'string' ? existing.s3_hash.toLowerCase() : null;
  if (actualHash !== expectedHash) {
    throw new Error(`${label}: existing attachment blob hash mismatch`);
  }
}
