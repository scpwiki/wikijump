function invalidOutput(detail) {
  throw new Error(`invalid category precreate output: ${detail}`);
}

export function parsePrecreatedCategoryIds(output) {
  const text = String(output ?? '').trim();
  if (text.length === 0) invalidOutput('empty result');

  let rows;
  try {
    rows = JSON.parse(text);
  } catch {
    invalidOutput('malformed JSON');
  }
  if (!Array.isArray(rows)) invalidOutput('expected a JSON array');

  const ids = new Map();
  for (const [index, row] of rows.entries()) {
    if (row === null || typeof row !== 'object' || Array.isArray(row)) {
      invalidOutput(`row ${index} must be an object`);
    }
    const keys = Object.keys(row).sort();
    if (keys.length !== 2 || keys[0] !== 'category_id' || keys[1] !== 'slug') {
      invalidOutput(`row ${index} has unexpected fields`);
    }
    if (typeof row.slug !== 'string') {
      invalidOutput(`row ${index} slug must be a string`);
    }
    if (typeof row.category_id !== 'string' || !/^[1-9][0-9]*$/u.test(row.category_id)) {
      invalidOutput(`row ${index} category_id must be a positive integer string`);
    }
    const categoryId = Number(row.category_id);
    if (!Number.isSafeInteger(categoryId)) {
      invalidOutput(`row ${index} category_id exceeds the safe integer range`);
    }
    if (ids.has(row.slug)) {
      invalidOutput(`row ${index} duplicates slug ${JSON.stringify(row.slug)}`);
    }
    ids.set(row.slug, categoryId);
  }
  return ids;
}
