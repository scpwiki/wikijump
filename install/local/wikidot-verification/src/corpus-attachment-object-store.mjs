import crypto from 'node:crypto';

const SERVICE = 's3';
const TERMINATOR = 'aws4_request';
const SHA512_RE = /^[0-9a-f]{128}$/u;

function assertNonEmptyString(value, name) {
  if (typeof value !== 'string' || value.length === 0) {
    throw new TypeError(`${name} must be a non-empty string`);
  }
}

function assertBoolean(value, name) {
  if (typeof value !== 'boolean') {
    throw new TypeError(`${name} must be a boolean`);
  }
}

function assertObjectKey(key) {
  if (typeof key !== 'string' || !SHA512_RE.test(key)) {
    throw new TypeError('object key must be a lowercase sha512 hex string');
  }
}

function sha256Hex(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function hmac(key, value, encoding = undefined) {
  return crypto.createHmac('sha256', key).update(value).digest(encoding);
}

function amzTimestamp(now) {
  const value = now.toISOString().replaceAll(/[:-]/gu, '').replace(/\.\d{3}Z$/u, 'Z');
  return { amzDate: value, dateStamp: value.slice(0, 8) };
}

function encodePathSegment(value) {
  return encodeURIComponent(value).replaceAll('%7E', '~');
}

function encodedObjectPath({ endpointUrl, bucket, key, pathStyle }) {
  const basePath = endpointUrl.pathname.replace(/\/+$/u, '');
  const encodedKey = key.split('/').map(encodePathSegment).join('/');
  if (pathStyle) {
    return `${basePath}/${encodePathSegment(bucket)}/${encodedKey}`;
  }
  return `${basePath}/${encodedKey}`;
}

function objectUrl({ endpointUrl, bucket, key, pathStyle }) {
  const url = new URL(endpointUrl.href);
  if (!pathStyle) {
    url.hostname = `${bucket}.${url.hostname}`;
  }
  url.pathname = encodedObjectPath({ endpointUrl, bucket, key, pathStyle });
  return url;
}

function canonicalHeaderValue(value) {
  return String(value).trim().replaceAll(/\s+/gu, ' ');
}

function signedHeaders(headers) {
  return Object.entries(headers)
    .map(([name, value]) => [name.toLowerCase(), canonicalHeaderValue(value)])
    .sort(([left], [right]) => left.localeCompare(right));
}

function signingKey({ secretAccessKey, dateStamp, region }) {
  const dateKey = hmac(`AWS4${secretAccessKey}`, dateStamp);
  const regionKey = hmac(dateKey, region);
  const serviceKey = hmac(regionKey, SERVICE);
  return hmac(serviceKey, TERMINATOR);
}

function signedRequestHeaders({
  method,
  url,
  accessKeyId,
  secretAccessKey,
  region,
  now,
  payloadHash,
  headers = {},
}) {
  const { amzDate, dateStamp } = amzTimestamp(now);
  const baseHeaders = {
    host: url.host,
    'x-amz-content-sha256': payloadHash,
    'x-amz-date': amzDate,
    ...headers,
  };
  const sortedHeaders = signedHeaders(baseHeaders);
  const canonicalHeaders = sortedHeaders.map(([name, value]) => `${name}:${value}\n`).join('');
  const signedHeaderNames = sortedHeaders.map(([name]) => name).join(';');
  const canonicalRequest = [
    method,
    url.pathname,
    url.searchParams.toString(),
    canonicalHeaders,
    signedHeaderNames,
    payloadHash,
  ].join('\n');
  const credentialScope = `${dateStamp}/${region}/${SERVICE}/${TERMINATOR}`;
  const stringToSign = [
    'AWS4-HMAC-SHA256',
    amzDate,
    credentialScope,
    sha256Hex(canonicalRequest),
  ].join('\n');
  const signature = hmac(
    signingKey({ secretAccessKey, dateStamp, region }),
    stringToSign,
    'hex',
  );

  return {
    ...baseHeaders,
    authorization: `AWS4-HMAC-SHA256 Credential=${accessKeyId}/${credentialScope}, SignedHeaders=${signedHeaderNames}, Signature=${signature}`,
  };
}

function responseHeader(response, name) {
  if (typeof response.headers?.get === 'function') {
    return response.headers.get(name);
  }
  return response.headers?.[name] ?? response.headers?.[name.toLowerCase()] ?? null;
}

function responseSize(response, key) {
  const value = responseHeader(response, 'content-length');
  if (value === null || value === undefined) {
    throw new Error(`HEAD response for ${key} did not include content-length`);
  }
  const size = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(size) || size < 0 || String(size) !== String(value).trim()) {
    throw new Error(`HEAD response for ${key} included invalid content-length ${value}`);
  }
  return size;
}

function s3Error(response, action, key) {
  return new Error(`${action} ${key} failed with HTTP ${response.status}`);
}

export function createHttpObjectStoreClient({
  endpoint,
  bucket,
  accessKeyId,
  secretAccessKey,
  region = 'local',
  pathStyle = true,
  fetchImpl = globalThis.fetch,
  now = () => new Date(),
} = {}) {
  assertNonEmptyString(endpoint, 'endpoint');
  assertNonEmptyString(bucket, 'bucket');
  assertNonEmptyString(accessKeyId, 'accessKeyId');
  assertNonEmptyString(secretAccessKey, 'secretAccessKey');
  assertNonEmptyString(region, 'region');
  assertBoolean(pathStyle, 'pathStyle');
  if (typeof fetchImpl !== 'function') {
    throw new TypeError('fetchImpl must be a function');
  }
  if (typeof now !== 'function') {
    throw new TypeError('now must be a function');
  }

  const endpointUrl = new URL(endpoint);
  if (!['http:', 'https:'].includes(endpointUrl.protocol)) {
    throw new TypeError('endpoint must use http or https');
  }

  async function request(method, key, { body = null, contentType = null } = {}) {
    assertObjectKey(key);
    const url = objectUrl({ endpointUrl, bucket, key, pathStyle });
    const payloadHash = body === null ? sha256Hex('') : sha256Hex(body);
    const headers = {};
    if (contentType !== null) headers['content-type'] = contentType;
    if (body !== null) headers['content-length'] = String(body.byteLength);
    const signed = signedRequestHeaders({
      method,
      url,
      accessKeyId,
      secretAccessKey,
      region,
      now: now(),
      payloadHash,
      headers,
    });
    return await fetchImpl(url, { method, headers: signed, body });
  }

  return Object.freeze({
    endpoint: endpointUrl.href,
    bucket,
    region,
    pathStyle,
    async headObject(key) {
      const response = await request('HEAD', key);
      if (response.status === 404) return { exists: false };
      if (response.status !== 200) throw s3Error(response, 'HEAD', key);
      return {
        exists: true,
        size: responseSize(response, key),
        contentType: responseHeader(response, 'content-type'),
      };
    },
    async putObject(key, bytes, { contentType = 'application/octet-stream' } = {}) {
      const body = Buffer.isBuffer(bytes) ? bytes : Buffer.from(bytes);
      const response = await request('PUT', key, { body, contentType });
      if (![200, 201, 204].includes(response.status)) {
        throw s3Error(response, 'PUT', key);
      }
    },
  });
}
