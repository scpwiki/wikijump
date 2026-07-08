const SHA256_RE = /^[0-9a-f]{64}$/u;
const S3_HASH_RE = /^[0-9a-f]{128}$/u;

const DEFAULT_REVISION_COMMENTS = 'local scp-wiki mirror attachment import from scp-wiki-translation corpus';

function assertSafeInteger(value, label) {
  if (!Number.isSafeInteger(value)) {
    throw new Error(`${label} must be a safe integer`);
  }
}

function assertAttachment(attachment, index) {
  if (attachment === null || typeof attachment !== 'object' || Array.isArray(attachment)) {
    throw new Error(`attachments[${index}] must be an object`);
  }
  for (const field of ['fullname', 'filename', 'sha256', 's3_key_hex']) {
    if (typeof attachment[field] !== 'string' || attachment[field].length === 0) throw new Error(`attachments[${index}].${field} must be a non-empty string`);
  }
  if (!SHA256_RE.test(attachment.sha256)) {
    throw new Error(`attachments[${index}].sha256 must be a lowercase sha256 hex string`);
  }
  if (!S3_HASH_RE.test(attachment.s3_key_hex)) {
    throw new Error(`attachments[${index}].s3_key_hex must be a lowercase sha512 hex string`);
  }
  if (!Number.isSafeInteger(attachment.size) || attachment.size < 0) {
    throw new Error(`attachments[${index}].size must be a non-negative safe integer`);
  }
  if (attachment.mime !== undefined && attachment.mime !== null && (typeof attachment.mime !== 'string' || attachment.mime.length === 0)) {
    throw new Error(`attachments[${index}].mime must be a non-empty string when set`);
  }
}

function sqlString(value) {
  return `'${value.replaceAll("'", "''")}'`;
}

function sqlNullableString(value) {
  return value === undefined || value === null ? 'NULL::text' : `${sqlString(value)}::text`;
}

function sqlBigint(value) {
  return `${value}::bigint`;
}

function plannedAttachmentValues(attachments, revisionComments) {
  if (attachments.length === 0) {
    return `SELECT NULL::integer AS row_index, NULL::text AS fullname, NULL::text AS filename, NULL::text AS sha256,
    NULL::bigint AS size, NULL::bytea AS s3_hash, NULL::text AS mime, NULL::text AS revision_comments
  WHERE false`;
  }

  const rows = attachments.map((attachment, index) => `(${index}::integer, ${sqlString(attachment.fullname)}::text, ${sqlString(attachment.filename)}::text, ${sqlString(attachment.sha256)}::text, ${sqlBigint(attachment.size)}, decode(${sqlString(attachment.s3_key_hex)}, 'hex'), ${sqlNullableString(attachment.mime)}, ${sqlString(revisionComments)}::text)`);
  return `SELECT *
  FROM (VALUES
    ${rows.join(',\n    ')}
  ) AS v(row_index, fullname, filename, sha256, size, s3_hash, mime, revision_comments)`;
}

export function buildAttachmentStagingSql({
  siteId,
  attachments,
  actorUserId,
  revisionComments = DEFAULT_REVISION_COMMENTS,
}) {
  assertSafeInteger(siteId, 'siteId');
  assertSafeInteger(actorUserId, 'actorUserId');
  if (!Array.isArray(attachments)) {
    throw new Error('attachments must be an array');
  }
  if (typeof revisionComments !== 'string' || revisionComments.length === 0) {
    throw new Error('revisionComments must be a non-empty string');
  }
  attachments.forEach(assertAttachment);

  return `WITH planned_attachments AS (
  ${plannedAttachmentValues(attachments, revisionComments)}
),
page_match AS (
  SELECT pa.*, p.page_id
  FROM planned_attachments pa
  LEFT JOIN page p
    ON p.site_id = ${sqlBigint(siteId)}
   AND p.slug = pa.fullname
   AND p.deleted_at IS NULL
),
active_file_matches AS (
  SELECT pm.row_index, count(f.file_id)::integer AS active_file_count, min(f.file_id) AS file_id
  FROM page_match pm
  LEFT JOIN file f
    ON f.site_id = ${sqlBigint(siteId)}
   AND f.page_id = pm.page_id
   AND f.name = pm.filename
   AND f.deleted_at IS NULL
  GROUP BY pm.row_index
),
latest_file_revisions AS (
  SELECT DISTINCT ON (fr.file_id) fr.file_id, fr.revision_id, fr.revision_number, fr.s3_hash, fr.size
  FROM file_revision fr
  JOIN active_file_matches af
    ON af.file_id = fr.file_id
  ORDER BY fr.file_id, fr.revision_number DESC, fr.revision_id DESC
),
classified AS (
  SELECT
    pm.row_index,
    pm.fullname,
    pm.filename,
    pm.sha256,
    pm.size,
    pm.s3_hash,
    COALESCE(pm.mime, 'application/octet-stream') AS mime,
    pm.revision_comments,
    pm.page_id,
    af.file_id,
    CASE
      WHEN pm.page_id IS NULL THEN 'fail_closed'
      WHEN bb.s3_hash IS NOT NULL THEN 'fail_closed'
      WHEN af.active_file_count > 1 THEN 'fail_closed'
      WHEN af.active_file_count = 1 AND lfr.file_id IS NULL THEN 'fail_closed'
      WHEN af.active_file_count = 1 AND lfr.size = pm.size AND lfr.s3_hash = pm.s3_hash THEN 'skip_existing'
      WHEN af.active_file_count = 1 THEN 'fail_closed'
      ELSE 'insert'
    END AS action,
    CASE
      WHEN pm.page_id IS NULL THEN 'missing_page'
      WHEN bb.s3_hash IS NOT NULL THEN 'blob_blacklisted'
      WHEN af.active_file_count > 1 THEN 'active_name_conflict'
      WHEN af.active_file_count = 1 AND lfr.file_id IS NULL THEN 'existing_missing_revision'
      WHEN af.active_file_count = 1 AND lfr.size = pm.size AND lfr.s3_hash = pm.s3_hash THEN NULL
      WHEN af.active_file_count = 1 THEN 'existing_mismatch'
      ELSE NULL
    END AS reason,
    CASE
      WHEN pm.page_id IS NOT NULL AND bb.s3_hash IS NULL AND af.active_file_count = 0 THEN 0
      ELSE NULL
    END AS revision_number
  FROM page_match pm
  JOIN active_file_matches af
    ON af.row_index = pm.row_index
  LEFT JOIN latest_file_revisions lfr
    ON lfr.file_id = af.file_id
  LEFT JOIN blob_blacklist bb
    ON bb.s3_hash = pm.s3_hash
),
staged_file_rows AS (
  SELECT row_index, ${sqlBigint(siteId)} AS site_id, page_id, filename AS name, false AS from_wikidot
  FROM classified
  WHERE action = 'insert'
),
staged_first_revisions AS (
  SELECT
    row_index,
    'create'::text AS revision_type,
    0::integer AS revision_number,
    page_id,
    ${sqlBigint(siteId)} AS site_id,
    ${sqlBigint(actorUserId)} AS user_id,
    filename AS name,
    s3_hash,
    mime,
    size,
    ARRAY['page', 'name', 'blob', 'mime']::text[] AS changes,
    revision_comments AS comments,
    ARRAY[]::text[] AS hidden
  FROM classified
  WHERE action = 'insert'
)
SELECT
  c.row_index, c.fullname, c.filename, c.action,
  COALESCE(c.reason, '') AS reason,
  COALESCE(c.page_id::text, '') AS page_id,
  COALESCE(c.file_id::text, '') AS file_id,
  COALESCE(c.revision_number::text, '') AS revision_number
FROM classified c
LEFT JOIN staged_file_rows sfr
  ON sfr.row_index = c.row_index
LEFT JOIN staged_first_revisions sfrv
  ON sfrv.row_index = c.row_index
ORDER BY c.row_index;`;
}

function parseNullableInteger(value, label, lineNumber) {
  if (value === '') return null;
  if (!/^-?\d+$/u.test(value)) {
    throw new Error(`line ${lineNumber}: ${label} must be an integer or empty`);
  }
  return Number(value);
}

export function parseAttachmentStagingResults(output) {
  const summary = { total: 0, insert: 0, skip_existing: 0, fail_closed: 0 };
  const rows = [];
  const lines = output.split(/\r?\n/u).map((line) => line.trim()).filter(Boolean);

  lines.forEach((line, index) => {
    const lineNumber = index + 1;
    const parts = line.split('|');
    if (parts.length !== 8) {
      throw new Error(`line ${lineNumber}: expected 8 pipe-delimited fields, got ${parts.length}`);
    }
    const [rowIndex, fullname, filename, action, reason, pageId, fileId, revisionNumber] = parts;
    if (!['insert', 'skip_existing', 'fail_closed'].includes(action)) {
      throw new Error(`line ${lineNumber}: unknown action ${action}`);
    }

    summary.total += 1;
    summary[action] += 1;
    rows.push({
      row_index: parseNullableInteger(rowIndex, 'row_index', lineNumber),
      fullname,
      filename,
      action,
      reason: reason === '' ? null : reason,
      page_id: parseNullableInteger(pageId, 'page_id', lineNumber),
      file_id: parseNullableInteger(fileId, 'file_id', lineNumber),
      revision_number: parseNullableInteger(revisionNumber, 'revision_number', lineNumber),
    });
  });

  return { summary, rows };
}
