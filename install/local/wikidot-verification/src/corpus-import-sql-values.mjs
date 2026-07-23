export function sqlQuote(value) {
  if (value === null || value === undefined) return 'NULL';
  return `'${String(value).replaceAll("'", "''")}'`;
}

export function sqlTimestamp(value) {
  if (value === null || value === undefined || value === '') return 'NULL';
  return `TIMESTAMPTZ ${sqlQuote(value)}`;
}

export function sqlInt(value) {
  if (!Number.isInteger(value)) throw new Error(`expected integer, got ${value}`);
  return String(value);
}

export function sqlByteaFromHex(hex) {
  if (!/^[0-9a-f]{64}$/iu.test(hex)) throw new Error(`expected sha256 hex, got ${hex}`);
  return `decode(${sqlQuote(hex.toLowerCase())}, 'hex')`;
}

export function sqlTextHash(hex) {
  if (!/^[0-9a-f]{32}$/iu.test(hex)) throw new Error(`expected 16-byte text hash hex, got ${hex}`);
  return `decode(${sqlQuote(hex.toLowerCase())}, 'hex')`;
}

export function sqlTextFromBase64(value) {
  return `convert_from(decode(${sqlQuote(Buffer.from(value, 'utf8').toString('base64'))}, 'base64'), 'UTF8')`;
}

export function sqlTextArray(values) {
  return `ARRAY[${values.map((value) => sqlQuote(value)).join(',')}]::text[]`;
}
