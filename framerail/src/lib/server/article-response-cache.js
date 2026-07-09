import { createHash } from "node:crypto"

const ARTICLE_ROUTES = new Set(["/", "/[slug]/[...extra]"])
const PERMISSION_FENCE = "anonymous-page-view-v1"
const RESPONSE_CACHE_PREFIX = "framerail:article-response:v1"
const RESPONSE_TOKEN_PREFIX = "framerail:article-response-token:v1"
const SESSION_COOKIE = "wikijump_token"
export const ARTICLE_RESPONSE_CACHE_MAX_ENTRIES = 1024
export const ARTICLE_RESPONSE_CACHE_MAX_BYTES = 32 * 1024 * 1024
export const ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES = 1024 * 1024
export const ARTICLE_RESPONSE_CACHE_TTL_SECONDS = 60
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS = 5000
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES = 1024
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES = 8 * 1024 * 1024
export const PUBLIC_CONTENT_FENCE_PREFIX = "deepwell:public-content:site"
export const ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL =
  "wikijump:article-response-fence-invalidation:v1"

const utf8Hex = (value) => {
  return Buffer.from(value, "utf8").toString("hex")
}

const sha256Hex = (value) => {
  return createHash("sha256").update(value).digest("hex")
}

const hasSessionCookie = (cookieHeader) => {
  if (!cookieHeader) return false

  return cookieHeader
    .split(";")
    .map((cookie) => cookie.trim())
    .some(
      (cookie) => cookie === SESSION_COOKIE || cookie.startsWith(`${SESSION_COOKIE}=`)
    )
}

const isEmptyExtra = (extra) => {
  return extra === undefined || extra === null || extra === ""
}

const normalizeFenceVersion = (value) => {
  if (value === undefined || value === null) return "0"
  if (typeof value !== "string" || !/^\d+$/.test(value)) return null
  return value
}

const parsePermissionFence = (value) => {
  if (typeof value !== "string") return null
  const match = /^site=(\d+),user=(\d+)$/.exec(value)
  if (!match) return null

  return {
    sitePermissionFence: match[1],
    userPermissionFence: match[2]
  }
}

const reject = (reason) => {
  return { cacheable: false, reason }
}

export const canConsiderAnonymousArticleResponseCache = ({
  method,
  routeId,
  url,
  siteId,
  siteSlug,
  route,
  cookieHeader
}) => {
  if (method !== "GET") return reject("method")
  if (!ARTICLE_ROUTES.has(routeId ?? "")) return reject("route")
  if (url.search !== "") return reject("query")
  if (!Number.isInteger(siteId) || siteId <= 0) return reject("site-id")
  if (!siteSlug) return reject("site-slug")
  if (hasSessionCookie(cookieHeader)) return reject("session-cookie")
  if (!isEmptyExtra(route?.extra)) return reject("extra")

  return { cacheable: true }
}

export const buildAnonymousArticleResponseCacheMetadata = ({
  siteId,
  siteSlug,
  requestLocales,
  backendLocales,
  deepwellArticlePageCacheKey,
  publicContentFence = "0",
  permissionFence = PERMISSION_FENCE
}) => {
  if (!Number.isInteger(siteId) || siteId <= 0) return null
  if (!siteSlug) return null
  if (!Array.isArray(requestLocales) || !Array.isArray(backendLocales)) return null
  if (!deepwellArticlePageCacheKey) return null
  if (!publicContentFence || !permissionFence) return null

  return {
    siteId,
    siteSlug,
    requestLocales,
    backendLocales,
    deepwellArticlePageCacheKey,
    publicContentFence,
    permissionFence
  }
}

export const buildPublicContentFenceKey = (siteId) => {
  return `${PUBLIC_CONTENT_FENCE_PREFIX}:${siteId}:version`
}

export const buildAnonymousPermissionFenceKeys = (siteId) => {
  return {
    siteKey: `permission:site:${siteId}:version`,
    userKey: `permission:site:${siteId}:user:anonymous:version`
  }
}

export const buildAnonymousArticleResponseCacheFences = ({
  siteId,
  siteSlug,
  route,
  requestLocales,
  backendLocales,
  publicContentFence,
  permissionFence
}) => {
  if (!Number.isInteger(siteId) || siteId <= 0) return null
  if (!siteSlug) return null
  if (!Array.isArray(requestLocales) || !Array.isArray(backendLocales)) return null
  if (!publicContentFence || !permissionFence) return null

  return {
    siteId,
    siteSlug,
    route: route ?? null,
    requestLocales,
    backendLocales,
    publicContentFence,
    permissionFence
  }
}

export const readAnonymousArticleResponseCacheFences = async ({ store, siteId }) => {
  if (!store || !Number.isInteger(siteId) || siteId <= 0) return null

  try {
    const publicContentFenceKey = buildPublicContentFenceKey(siteId)
    const { siteKey, userKey } = buildAnonymousPermissionFenceKeys(siteId)
    const [publicContentFenceValue, sitePermissionFenceValue, userPermissionFenceValue] =
      typeof store.mget === "function"
        ? await store.mget([publicContentFenceKey, siteKey, userKey])
        : [
            await store.get(publicContentFenceKey),
            await store.get(siteKey),
            await store.get(userKey)
          ]
    const publicContentFence = normalizeFenceVersion(publicContentFenceValue)
    const sitePermissionFence = normalizeFenceVersion(sitePermissionFenceValue)
    const userPermissionFence = normalizeFenceVersion(userPermissionFenceValue)

    if (
      publicContentFence === null ||
      sitePermissionFence === null ||
      userPermissionFence === null
    ) {
      return null
    }

    return {
      publicContentFence,
      permissionFence: `site=${sitePermissionFence},user=${userPermissionFence}`
    }
  } catch {
    return null
  }
}

export const createMemoryArticleResponseFenceCache = ({ store, subscriber } = {}) => {
  const sites = new Map()
  const hotCaches = new Set()
  let trusted = false
  let fenceRevision = 0

  const clearHotResponses = () => {
    for (const hotCache of hotCaches) {
      hotCache?.clear?.()
    }
  }

  const poison = () => {
    fenceRevision += 1
    trusted = false
    sites.clear()
    clearHotResponses()
  }

  const seedSite = async (siteId) => {
    const seedRevision = fenceRevision
    const fences = await readAnonymousArticleResponseCacheFences({ store, siteId })
    const permission = parsePermissionFence(fences?.permissionFence)
    if (!fences || !permission) return null
    if (seedRevision !== fenceRevision) return sites.get(siteId) ?? null

    const site = {
      publicContentFence: fences.publicContentFence,
      sitePermissionFence: permission.sitePermissionFence,
      userPermissionFence: permission.userPermissionFence
    }
    sites.set(siteId, site)
    return site
  }

  const isGap = (current, next) => {
    if (current === undefined) return true
    try {
      return BigInt(next) > BigInt(current) + 1n
    } catch {
      return true
    }
  }

  const messageSiteId = (value) => {
    return Number.isInteger(value) && value > 0 ? value : null
  }

  const applyPublicContentMessage = (message) => {
    const siteId = messageSiteId(message.site_id)
    const version = normalizeFenceVersion(message.version)
    if (!siteId || version === null) return false

    fenceRevision += 1
    const site = sites.get(siteId)
    if (!site) {
      clearHotResponses()
      return true
    }
    if (BigInt(version) <= BigInt(site.publicContentFence)) return true
    if (isGap(site.publicContentFence, version)) clearHotResponses()
    site.publicContentFence = version
    clearHotResponses()
    return true
  }

  const applyAnonymousPermissionMessage = (message) => {
    const siteId = messageSiteId(message.site_id)
    const siteVersion = normalizeFenceVersion(message.site_version)
    const userVersion = normalizeFenceVersion(message.user_version)
    if (!siteId || siteVersion === null || userVersion === null) return false

    fenceRevision += 1
    const site = sites.get(siteId)
    if (!site) {
      clearHotResponses()
      return true
    }
    const siteAdvanced = BigInt(siteVersion) > BigInt(site.sitePermissionFence)
    const userAdvanced = BigInt(userVersion) > BigInt(site.userPermissionFence)
    if (!siteAdvanced && !userAdvanced) return true
    if (
      isGap(site.sitePermissionFence, siteVersion) ||
      isGap(site.userPermissionFence, userVersion)
    ) {
      clearHotResponses()
    }
    site.sitePermissionFence = siteVersion
    site.userPermissionFence = userVersion
    clearHotResponses()
    return true
  }

  const ignoreNonAnonymousPermissionMessage = (message) => {
    const siteId = messageSiteId(message.site_id)
    const version = normalizeFenceVersion(message.version)
    return Boolean(
      siteId && Number.isInteger(message.user_id) && message.user_id > 0 && version
    )
  }

  const applyMessage = (payload) => {
    let message
    try {
      message = JSON.parse(payload)
    } catch {
      poison()
      return false
    }
    if (!message || typeof message !== "object") {
      poison()
      return false
    }

    let applied = false
    if (message.type === "public-content") {
      applied = applyPublicContentMessage(message)
    } else if (message.type === "anonymous-permission") {
      applied = applyAnonymousPermissionMessage(message)
    } else if (message.type === "user-permission") {
      applied = ignoreNonAnonymousPermissionMessage(message)
    }
    if (!applied) poison()
    return applied
  }

  let subscription = null
  const api = {
    attachHotCache(hotCache) {
      if (hotCache) hotCaches.add(hotCache)
    },

    async readFences({ siteId }) {
      if (!trusted) {
        return readAnonymousArticleResponseCacheFences({ store, siteId })
      }

      const site = sites.get(siteId) ?? (await seedSite(siteId))
      if (!site) return null

      return {
        publicContentFence: site.publicContentFence,
        permissionFence: `site=${site.sitePermissionFence},user=${site.userPermissionFence}`
      }
    },

    areFencesCurrent({ siteId, publicContentFence, permissionFence }) {
      if (!trusted) return null
      const site = sites.get(siteId)
      if (!site) return null

      return (
        site.publicContentFence === publicContentFence &&
        `site=${site.sitePermissionFence},user=${site.userPermissionFence}` ===
          permissionFence
      )
    },

    canValidateFencesLocally({ siteId }) {
      return trusted && sites.has(siteId)
    },

    markSubscribedForTest: async () => {
      trusted = true
    },

    markDisconnectedForTest: () => {
      poison()
    },

    applyMessageForTest: async (payload) => {
      applyMessage(payload)
    },

    close() {
      subscription?.close?.()
    }
  }

  subscription = subscriber?.subscribe?.({
    channel: ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL,
    onSubscribed: () => {
      trusted = true
    },
    onMessage: applyMessage,
    onDisconnect: poison,
    onMalformed: poison
  })

  return api
}

export const buildAnonymousArticleResponseCacheKey = (metadata) => {
  return [
    RESPONSE_CACHE_PREFIX,
    `site=${metadata.siteId}`,
    `slug=${utf8Hex(metadata.siteSlug)}`,
    `requestLocales=${utf8Hex(metadata.requestLocales.join(","))}`,
    `backendLocales=${utf8Hex(metadata.backendLocales.join(","))}`,
    `content=${metadata.publicContentFence}`,
    `permission=${metadata.permissionFence}`,
    `deepwell=${sha256Hex(metadata.deepwellArticlePageCacheKey)}`
  ].join(":")
}

const normalizeRouteForToken = (route) => {
  if (!route) return { slug: "", extra: "" }
  return {
    slug: route.slug ?? "",
    extra: route.extra ?? ""
  }
}

export const buildAnonymousArticleResponseTokenKey = (metadata) => {
  const route = normalizeRouteForToken(metadata.route)

  return [
    RESPONSE_TOKEN_PREFIX,
    `site=${metadata.siteId}`,
    `slug=${utf8Hex(metadata.siteSlug)}`,
    `route=${sha256Hex(JSON.stringify(route))}`,
    `requestLocales=${utf8Hex(metadata.requestLocales.join(","))}`,
    `backendLocales=${utf8Hex(metadata.backendLocales.join(","))}`,
    `content=${metadata.publicContentFence}`,
    `permission=${utf8Hex(metadata.permissionFence)}`
  ].join(":")
}

const normalizedHeaderEntries = (headers) => {
  return [...headers.entries()]
    .filter(([name]) => name.toLowerCase() !== "set-cookie")
    .sort(([left], [right]) => left.localeCompare(right))
}

export const serializeArticleResponseForCache = async (response) => {
  return {
    status: response.status,
    headers: normalizedHeaderEntries(response.headers),
    body: await response.clone().text()
  }
}

const serializedByteLength = (value) => {
  return Buffer.byteLength(value, "utf8")
}

export const createMemoryArticleResponseCacheStore = ({
  now = () => Date.now(),
  maxEntries = ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_CACHE_MAX_BYTES
} = {}) => {
  const entries = new Map()
  let totalBytes = 0
  const maxEntryCount =
    Number.isInteger(maxEntries) && maxEntries > 0
      ? maxEntries
      : ARTICLE_RESPONSE_CACHE_MAX_ENTRIES
  const maxTotalBytes =
    Number.isInteger(maxBytes) && maxBytes > 0
      ? maxBytes
      : ARTICLE_RESPONSE_CACHE_MAX_BYTES

  const deleteEntry = (key) => {
    const entry = entries.get(key)
    if (!entry) return
    totalBytes -= entry.bytes
    entries.delete(key)
  }

  const pruneExpired = (nowMs) => {
    for (const [key, entry] of entries) {
      if (entry.expiresAt <= nowMs) {
        deleteEntry(key)
      }
    }
  }

  const pruneOverflow = () => {
    while (entries.size > maxEntryCount || totalBytes > maxTotalBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

  return {
    async get(key) {
      const entry = entries.get(key)
      if (!entry) return null

      if (entry.expiresAt <= now()) {
        deleteEntry(key)
        return null
      }

      return entry.value
    },

    async set(key, value, ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS) {
      const nowMs = now()
      const expiresAt = nowMs + ttlSeconds * 1000

      pruneExpired(nowMs)
      if (expiresAt <= nowMs) {
        deleteEntry(key)
        return false
      }

      const bytes = serializedByteLength(value)
      deleteEntry(key)
      if (bytes > maxTotalBytes) return false

      entries.set(key, {
        value,
        expiresAt,
        bytes
      })
      totalBytes += bytes
      pruneOverflow()
      return entries.has(key)
    },

    size() {
      return entries.size
    }
  }
}

const isHeaderPair = (value) => {
  return (
    Array.isArray(value) &&
    value.length === 2 &&
    typeof value[0] === "string" &&
    typeof value[1] === "string"
  )
}

const isCachedArticleResponse = (value) => {
  return (
    value !== null &&
    typeof value === "object" &&
    Number.isInteger(value.status) &&
    value.status >= 200 &&
    value.status <= 599 &&
    Array.isArray(value.headers) &&
    value.headers.every(isHeaderPair) &&
    typeof value.body === "string"
  )
}

export const normalizeCachedArticleResponseEntry = (value) => {
  if (!isCachedArticleResponse(value)) return null

  return {
    status: value.status,
    headers: value.headers.map(([name, headerValue]) => [name, headerValue]),
    body: value.body
  }
}

const freezeHeaderEntries = (headers) => {
  return Object.freeze(headers.map(([name, value]) => Object.freeze([name, value])))
}

const normalizeCachedArticleResponseReplay = (entry, replay) => {
  const status = replay?.status ?? entry.status
  const headers = replay?.headers ?? entry.headers
  const bodyBuffer = replay?.bodyBuffer

  if (status !== entry.status) return null
  if (!Array.isArray(headers) || !headers.every(isHeaderPair)) return null
  if (bodyBuffer !== undefined && !Buffer.isBuffer(bodyBuffer)) return null
  const replayBodyBuffer =
    bodyBuffer === undefined ? Buffer.from(entry.body, "utf8") : Buffer.from(bodyBuffer)

  return Object.freeze({
    status,
    headers: freezeHeaderEntries(headers.map(([name, value]) => [name, value])),
    bodyBuffer: replayBodyBuffer,
    finalHeaders: replay?.finalHeaders === true
  })
}

const copyCachedArticleResponseReplay = (replay) => {
  return Object.freeze({
    status: replay.status,
    headers: replay.headers,
    bodyBuffer: Buffer.from(replay.bodyBuffer),
    finalHeaders: replay.finalHeaders
  })
}

const copyCachedArticleResponseEntry = (entry) => {
  const copy = {
    status: entry.status,
    headers: entry.headers.map(([name, value]) => [name, value]),
    body: entry.body
  }
  if (Buffer.isBuffer(entry.bodyBuffer)) {
    copy.bodyBuffer = Buffer.from(entry.bodyBuffer)
  }
  return copy
}

const cachedArticleResponseEntryByteLength = (key, entry) => {
  let bytes = serializedByteLength(key) + 8
  bytes += serializedByteLength(entry.body)
  for (const [name, value] of entry.headers) {
    bytes += serializedByteLength(name) + serializedByteLength(value) + 4
  }
  return bytes
}

export const createLocalArticleResponseHotCache = ({
  now = () => Date.now(),
  ttlMs = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS,
  maxEntries = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES
} = {}) => {
  const entries = new Map()
  let totalBytes = 0
  const maxEntryCount =
    Number.isInteger(maxEntries) && maxEntries > 0
      ? maxEntries
      : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES
  const maxTotalBytes =
    Number.isInteger(maxBytes) && maxBytes > 0
      ? maxBytes
      : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES
  const entryTtlMs =
    Number.isFinite(ttlMs) && ttlMs > 0 ? ttlMs : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS

  const deleteEntry = (key) => {
    const entry = entries.get(key)
    if (!entry) return
    totalBytes -= entry.bytes
    entries.delete(key)
  }

  const pruneExpired = (nowMs) => {
    for (const [key, entry] of entries) {
      if (entry.expiresAt <= nowMs) {
        deleteEntry(key)
      }
    }
  }

  const pruneOverflow = () => {
    while (entries.size > maxEntryCount || totalBytes > maxTotalBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

  const getRecord = (key) => {
    const entry = entries.get(key)
    if (!entry) return null

    if (entry.expiresAt <= now()) {
      deleteEntry(key)
      return null
    }

    entries.delete(key)
    entries.set(key, entry)
    return entry
  }

  return {
    get(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return copyCachedArticleResponseEntry(entry.value)
    },

    getReplay(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return copyCachedArticleResponseReplay(entry.value.replay)
    },

    // Trusted fast-path callers pass this Buffer directly to ServerResponse.end.
    // They must not mutate the shared bodyBuffer.
    getSharedReplayForInternalUse(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return entry.value.replay
    },

    set(key, value, { replay } = {}) {
      if (typeof key !== "string" || key.length === 0) return false

      const normalized = normalizeCachedArticleResponseEntry(value)
      if (!normalized) {
        deleteEntry(key)
        return false
      }
      const bodyBuffer = Buffer.from(normalized.body, "utf8")
      const preparedReplay = normalizeCachedArticleResponseReplay(
        { ...normalized, bodyBuffer },
        replay
      )
      if (!preparedReplay) {
        deleteEntry(key)
        return false
      }

      const nowMs = now()
      const expiresAt = nowMs + entryTtlMs
      const bytes = cachedArticleResponseEntryByteLength(key, normalized)

      pruneExpired(nowMs)
      deleteEntry(key)
      if (bytes > maxTotalBytes) return false

      const cachedValue = copyCachedArticleResponseEntry({
        ...normalized,
        bodyBuffer
      })
      cachedValue.replay = preparedReplay
      entries.set(key, {
        value: cachedValue,
        expiresAt,
        bytes
      })
      totalBytes += bytes
      pruneOverflow()
      return entries.has(key)
    },

    size() {
      return entries.size
    },

    clear() {
      entries.clear()
      totalBytes = 0
    }
  }
}

export const deserializeCachedArticleResponse = (value) => {
  const entry = normalizeCachedArticleResponseEntry(value)
  if (!entry) return null

  return new Response(entry.body, {
    status: entry.status,
    headers: entry.headers
  })
}

export const readCachedArticleResponseEntry = async (store, key) => {
  try {
    const cached = await store.get(key)
    if (typeof cached !== "string") return null

    return normalizeCachedArticleResponseEntry(JSON.parse(cached))
  } catch {
    return null
  }
}

export const readCachedArticleResponse = async (store, key) => {
  const entry = await readCachedArticleResponseEntry(store, key)
  return deserializeCachedArticleResponse(entry)
}

export const writeCachedArticleResponse = async (
  store,
  key,
  entry,
  ttlSeconds,
  { maxSerializedBytes = ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES } = {}
) => {
  try {
    const serialized = JSON.stringify(entry)
    if (serializedByteLength(serialized) > maxSerializedBytes) return false

    return (await store.set(key, serialized, ttlSeconds)) !== false
  } catch {
    return false
  }
}

export const readAnonymousArticleResponseCache = async ({ store, metadata }) => {
  if (!metadata) return null
  return readCachedArticleResponse(store, buildAnonymousArticleResponseCacheKey(metadata))
}

export const readAnonymousArticleResponseCacheEntry = async ({ store, metadata }) => {
  if (!metadata) return null
  return readCachedArticleResponseEntry(
    store,
    buildAnonymousArticleResponseCacheKey(metadata)
  )
}

const isTokenValue = (value) => {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof value.articlePageCacheKey === "string" &&
    value.articlePageCacheKey.length > 0
  )
}

export const readAnonymousArticleResponseToken = async ({ store, tokenMetadata }) => {
  if (!tokenMetadata) return null

  try {
    const cached = await store.get(buildAnonymousArticleResponseTokenKey(tokenMetadata))
    if (typeof cached !== "string") return null

    const value = JSON.parse(cached)
    if (!isTokenValue(value)) return null

    return value.articlePageCacheKey
  } catch {
    return null
  }
}

export const writeAnonymousArticleResponseToken = async ({
  store,
  tokenMetadata,
  deepwellArticlePageCacheKey,
  ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS
}) => {
  if (!tokenMetadata || !deepwellArticlePageCacheKey) return false

  try {
    const currentFences = await readAnonymousArticleResponseCacheFences({
      store,
      siteId: tokenMetadata.siteId
    })
    if (
      currentFences?.publicContentFence !== tokenMetadata.publicContentFence ||
      currentFences?.permissionFence !== tokenMetadata.permissionFence
    ) {
      return false
    }

    const value = JSON.stringify({ articlePageCacheKey: deepwellArticlePageCacheKey })
    return (
      (await store.set(
        buildAnonymousArticleResponseTokenKey(tokenMetadata),
        value,
        ttlSeconds
      )) !== false
    )
  } catch {
    return false
  }
}

export const writeAnonymousArticleResponseCache = async ({
  store,
  metadata,
  response,
  ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS
}) => {
  if (!metadata) return false
  if (response.status !== 200) return false
  if (response.headers.has("set-cookie")) return false

  const entry = await serializeArticleResponseForCache(response)
  return writeCachedArticleResponse(
    store,
    buildAnonymousArticleResponseCacheKey(metadata),
    entry,
    ttlSeconds
  )
}
