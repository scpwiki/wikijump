export function parsePrecreatedCategoryIds(output) {
  const ids = new Map();
  if (!output.trim()) return ids;

  for (const line of output.split('\n')) {
    const [slugHex, categoryIdText, extra] = line.split('|');
    const categoryId = Number(categoryIdText);
    const validCategoryId = /^[1-9][0-9]*$/u.test(categoryIdText)
      && Number.isSafeInteger(categoryId);
    if (
      extra !== undefined
      || !/^(?:[0-9a-f]{2})*$/u.test(slugHex)
      || !validCategoryId
    ) {
      throw new Error(`invalid category precreate output: ${line}`);
    }
    ids.set(Buffer.from(slugHex, 'hex').toString('utf8'), categoryId);
  }

  return ids;
}
