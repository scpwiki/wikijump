function sqlQuote(value) {
  if (value === null || value === undefined) return 'NULL';
  return `'${String(value).replaceAll("'", "''")}'`;
}

function sqlInt(value) {
  if (!Number.isInteger(value)) throw new Error(`expected integer, got ${value}`);
  return String(value);
}

function parseNonNegativeIntegerField(value, context) {
  if (!/^(0|[1-9][0-9]*)$/.test(value)) {
    throw new Error(`invalid ${context}: ${value}`);
  }
  const number = Number(value);
  if (!Number.isSafeInteger(number)) {
    throw new Error(`invalid ${context}: ${value}`);
  }
  return number;
}

export function manifestRowsWithParents(rows) {
  return rows.filter((row) => typeof row.parent_fullname === 'string' && row.parent_fullname.length > 0);
}

export function buildParentLinkSql(args, rows) {
  const rowsWithParents = manifestRowsWithParents(rows);
  if (rowsWithParents.length === 0) return null;

  const values = rowsWithParents
    .map((row) => `(${sqlQuote(row.fullname)}, ${sqlQuote(row.parent_fullname)})`)
    .join(',\n');

  return `
WITH requested(child_slug, parent_slug) AS (
  VALUES
${values}
), resolved AS (
  SELECT
    requested.child_slug,
    requested.parent_slug,
    child.page_id AS child_page_id,
    parent.page_id AS parent_page_id
  FROM requested
  LEFT JOIN page child
    ON child.site_id = ${sqlInt(args.siteId)}
   AND child.slug = requested.child_slug
   AND child.deleted_at IS NULL
  LEFT JOIN page parent
    ON parent.site_id = ${sqlInt(args.siteId)}
   AND parent.slug = requested.parent_slug
   AND parent.deleted_at IS NULL
), linked AS (
  INSERT INTO page_parent (parent_page_id, child_page_id)
  SELECT parent_page_id, child_page_id
  FROM resolved
  WHERE parent_page_id IS NOT NULL
    AND child_page_id IS NOT NULL
  ON CONFLICT DO NOTHING
  RETURNING parent_page_id, child_page_id
)
SELECT
  (SELECT COUNT(*) FROM requested)::text || '|' ||
  (SELECT COUNT(*) FROM resolved WHERE parent_page_id IS NOT NULL AND child_page_id IS NOT NULL)::text || '|' ||
  (SELECT COUNT(*) FROM linked)::text || '|' ||
  (SELECT COUNT(*) FROM resolved WHERE parent_page_id IS NULL)::text || '|' ||
  (SELECT COUNT(*) FROM resolved WHERE child_page_id IS NULL)::text;
`;
}

export function buildParentLinkParentPagesSql(args, rows) {
  const rowsWithParents = manifestRowsWithParents(rows);
  if (rowsWithParents.length === 0) return null;

  const values = rowsWithParents
    .map((row) => `(${sqlQuote(row.fullname)}, ${sqlQuote(row.parent_fullname)})`)
    .join(',\n');

  return `
WITH requested(child_slug, parent_slug) AS (
  VALUES
${values}
)
SELECT DISTINCT parent.page_id::text || '|' || parent.page_category_id::text
FROM requested
JOIN page child
  ON child.site_id = ${sqlInt(args.siteId)}
 AND child.slug = requested.child_slug
 AND child.deleted_at IS NULL
JOIN page parent
  ON parent.site_id = ${sqlInt(args.siteId)}
 AND parent.slug = requested.parent_slug
 AND parent.deleted_at IS NULL
ORDER BY 1;
`;
}

export function parseParentLinkParentPages(output) {
  const trimmed = String(output ?? '').trim();
  if (trimmed.length === 0) return [];

  return trimmed
    .split('\n')
    .map((line) => {
      const fields = line.split('|');
      if (fields.length !== 2) {
        throw new Error(`invalid parent page link row: ${line}`);
      }
      const page_id = parseNonNegativeIntegerField(fields[0], 'parent page link row');
      const page_category_id = parseNonNegativeIntegerField(fields[1], 'parent page link row');
      return { page_id, page_category_id };
    });
}

export function parseParentLinkSummary(output) {
  const text = String(output ?? '').trim();
  const fields = text.split('|');
  if (fields.length !== 5) {
    throw new Error(`invalid parent link summary: ${output}`);
  }
  const [requestedText, readyText, insertedText, missingParentText, missingChildText] = fields;
  const summary = {
    parent_link_requested: parseNonNegativeIntegerField(requestedText, 'parent link summary requested'),
    parent_link_ready: parseNonNegativeIntegerField(readyText, 'parent link summary ready'),
    parent_link_inserted: parseNonNegativeIntegerField(insertedText, 'parent link summary inserted'),
    parent_link_missing_parent: parseNonNegativeIntegerField(missingParentText, 'parent link summary missing parent'),
    parent_link_missing_child: parseNonNegativeIntegerField(missingChildText, 'parent link summary missing child'),
  };
  return summary;
}
