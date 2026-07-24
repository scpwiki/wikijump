import crypto from 'node:crypto';
import dns from 'node:dns';
import fs from 'node:fs';
import http from 'node:http';
import https from 'node:https';

import {
  assertExistingAttachmentMatches,
  attachmentActorUserId,
} from './corpus-attachment-policy.mjs';
import { uploadPlannedAttachmentBlobs } from './corpus-attachment-direct-upload.mjs';
import { createHttpObjectStoreClient } from './corpus-attachment-object-store.mjs';
import { buildAttachmentStagingSql, parseAttachmentStagingResults } from './corpus-attachment-staging-sql.mjs';

const ATTACHMENT_IMPORT_COMMENTS = 'local scp-wiki mirror attachment import from scp-wiki-translation corpus';

function assertAttachmentSha(attachment) {
  if (typeof attachment.sha256 !== 'string' || !/^[0-9a-f]{64}$/u.test(attachment.sha256)) {
    throw new Error(`invalid attachment sha256 for ${attachment.filename ?? '<unknown>'}`);
  }
}

export function readCorpusAttachmentBytes(row, attachment) {
  if (attachment === null || typeof attachment !== 'object' || Array.isArray(attachment)) {
    throw new Error(`${row.fullname}: attachment entry must be an object`);
  }
  if (typeof attachment.filename !== 'string' || attachment.filename.length === 0) {
    throw new Error(`${row.fullname}: attachment filename must be a non-empty string`);
  }
  if (typeof attachment.file_path !== 'string' || attachment.file_path.length === 0) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment file_path must be a non-empty string`);
  }
  if (!Number.isSafeInteger(attachment.size) || attachment.size < 0) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment size must be a non-negative safe integer`);
  }
  assertAttachmentSha(attachment);

  const bytes = fs.readFileSync(attachment.file_path);
  const actualSha = crypto.createHash('sha256').update(bytes).digest('hex');
  if (actualSha !== attachment.sha256) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment sha256 mismatch: expected ${attachment.sha256}, got ${actualSha}`);
  }
  if (bytes.length !== attachment.size) {
    throw new Error(`${row.fullname}/${attachment.filename}: attachment size mismatch: expected ${attachment.size}, got ${bytes.length}`);
  }
  return bytes;
}

function putPresignedBytes(args, presignUrl, bytes, attachment) {
  const url = new URL(presignUrl);
  const client = url.protocol === 'https:' ? https : http;
  const aliases = args.presignHostAliases ?? new Map();
  const requestOptions = {
    method: 'PUT',
    hostname: url.hostname,
    port: url.port || (url.protocol === 'https:' ? 443 : 80),
    path: `${url.pathname}${url.search}`,
    headers: { 'content-length': bytes.byteLength },
    lookup(hostname, options, callback) {
      const alias = aliases.get(hostname.toLowerCase());
      if (alias) {
        const family = alias.includes(':') ? 6 : 4;
        if (options?.all) {
          callback(null, [{ address: alias, family }]);
        } else {
          callback(null, alias, family);
        }
        return;
      }
      dns.lookup(hostname, options, callback);
    },
  };

  return new Promise((resolve, reject) => {
    const request = client.request(requestOptions, (response) => {
      response.resume();
      response.on('end', () => resolve({ statusCode: response.statusCode ?? 0 }));
    });
    request.setTimeout(args.rpcTimeoutMs, () => {
      request.destroy(new Error(`${attachment.filename}: presigned PUT timed out after ${args.rpcTimeoutMs}ms`));
    });
    request.on('error', reject);
    request.end(bytes);
  });
}

async function uploadBlob({ args, attachment, bytes, actorUserId, rpc }) {
  const upload = await rpc(args, 'blob_upload', { user_id: actorUserId, blob_size: bytes.byteLength });
  const { statusCode } = await putPresignedBytes(args, upload.presign_url, bytes, attachment);
  if (statusCode < 200 || statusCode >= 300) {
    throw new Error(`${attachment.filename}: presigned PUT failed with status ${statusCode}`);
  }
  return upload.pending_blob_id;
}

async function createFile({ args, pageId, attachment, pendingBlobId, actorUserId, rpc }) {
  return await rpc(args, 'file_create', {
    site_id: args.siteId,
    page_id: pageId,
    name: attachment.filename,
    uploaded_blob_id: pendingBlobId,
    revision_comments: ATTACHMENT_IMPORT_COMMENTS,
    user_id: actorUserId,
    bypass_filter: true,
    ip_address: args.ipAddress,
  }, { siteId: args.siteId, pageRef: pageId });
}

export async function materializeCorpusRowAttachments({ args, row, pageId, getFile, rpc }) {
  const attachments = Array.isArray(row.attachments) ? row.attachments : [];
  if (args.skipAttachments) {
    return { attachments_requested: attachments.length, attachments_uploaded: 0, attachments_skipped_existing: 0, attachments_deferred: attachments.length };
  }
  if (args.attachmentCreateMode === 'direct') {
    return { attachments_requested: attachments.length, attachments_uploaded: 0, attachments_skipped_existing: 0 };
  }
  if (attachments.length === 0) {
    return { attachments_requested: 0, attachments_uploaded: 0, attachments_skipped_existing: 0 };
  }
  if (!args.sessionToken) {
    throw new Error(`${row.fullname}: attachment materialization requires DEEPWELL_SESSION_TOKEN`);
  }

  let uploaded = 0;
  let skippedExisting = 0;
  const actorUserId = attachmentActorUserId(args, row);
  for (const attachment of attachments) {
    const bytes = readCorpusAttachmentBytes(row, attachment);
    const existing = await getFile(args, pageId, attachment.filename);
    if (existing !== null) {
      assertExistingAttachmentMatches({ row, attachment, existing, bytes });
      skippedExisting += 1;
      continue;
    }
    const pendingBlobId = await uploadBlob({ args, attachment, bytes, actorUserId, rpc });
    await createFile({ args, pageId, attachment, pendingBlobId, actorUserId, rpc });
    uploaded += 1;
  }
  return { attachments_requested: attachments.length, attachments_uploaded: uploaded, attachments_skipped_existing: skippedExisting };
}

function createAttachmentObjectStore(args) {
  return createHttpObjectStoreClient({
    endpoint: args.attachmentS3Endpoint,
    bucket: args.attachmentS3Bucket,
    accessKeyId: args.attachmentS3AccessKeyId,
    secretAccessKey: args.attachmentS3SecretAccessKey,
    region: args.attachmentS3Region ?? 'local',
    pathStyle: args.attachmentS3PathStyle ?? true,
  });
}

export function summarizeCorpusAttachmentUpload(upload) {
  return {
    requested: upload.requested,
    uploaded: upload.uploaded,
    skipped_existing: upload.skipped_existing,
    failed: upload.failed,
    total_bytes: upload.total_bytes,
    uploaded_bytes: upload.uploaded_bytes,
  };
}

export async function uploadDirectCorpusAttachmentBlobs(args, directPlan) {
  if (directPlan.blobs.length === 0) {
    return { requested: 0, uploaded: 0, skipped_existing: 0, failed: 0, total_bytes: 0, uploaded_bytes: 0, results: [] };
  }
  return await uploadPlannedAttachmentBlobs({
    blobs: directPlan.blobs,
    objectStore: createAttachmentObjectStore(args),
    readBlobBytes: async (blob) => fs.readFileSync(blob.first_file_path),
  });
}

export async function commitDirectCorpusAttachmentStaging(args, sqlExecutor, directPlan) {
  const sql = buildAttachmentStagingSql({
    siteId: args.siteId,
    actorUserId: attachmentActorUserId(args),
    attachments: directPlan.attachments,
    revisionComments: ATTACHMENT_IMPORT_COMMENTS,
    commit: true,
  });
  return parseAttachmentStagingResults(await sqlExecutor.runSql(sql, { capture: true }));
}
