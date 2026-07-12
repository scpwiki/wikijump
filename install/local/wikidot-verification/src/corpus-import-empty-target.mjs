export async function assertEmptyDbImportTarget(args, sqlExecutor) {
  if (!args.assumeEmptyDbImport) return;
  if (!Number.isInteger(args.siteId)) {
    throw new Error(`expected integer site ID, got ${args.siteId}`);
  }

  const output = await sqlExecutor.runSql(`
SELECT page_id::text || '|' || slug
FROM page
WHERE site_id = ${args.siteId}
  AND deleted_at IS NULL
ORDER BY page_id
LIMIT 1;
`, { capture: true });
  if (!output) return;

  const [pageIdText, slug = ''] = output.split('|');
  throw new Error(`--assume-empty-db-import requires an empty active page set for site ${args.siteId}; found page ${pageIdText} (${slug})`);
}
