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

function createAuthTodoError() {
  return new Error('HTTP object-store access is guarded until authenticated AWS SigV4 HEAD/PUT support is implemented');
}

export function createHttpObjectStoreClient({
  endpoint,
  bucket,
  accessKeyId,
  secretAccessKey,
  region = 'local',
  pathStyle = true,
  fetchImpl = globalThis.fetch,
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

  const endpointUrl = new URL(endpoint);
  if (!['http:', 'https:'].includes(endpointUrl.protocol)) {
    throw new TypeError('endpoint must use http or https');
  }

  return Object.freeze({
    endpoint: endpointUrl.href,
    bucket,
    region,
    pathStyle,
    async headObject() {
      throw createAuthTodoError();
    },
    async putObject() {
      throw createAuthTodoError();
    },
  });
}
