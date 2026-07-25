import {
  ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL,
  hasValidArticleResponseCacheIdentity,
  PUBLIC_CONTENT_FENCE_PREFIX
} from "./shared.js"

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
  requestHost,
  route,
  requestLocales,
  backendLocales,
  publicContentFence,
  permissionFence
}) => {
  if (
    !hasValidArticleResponseCacheIdentity({
      siteId,
      siteSlug,
      requestHost,
      requestLocales,
      backendLocales
    })
  ) {
    return null
  }
  if (!publicContentFence || !permissionFence) return null

  return {
    siteId,
    siteSlug,
    requestHost,
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
