import {
  hasValidArticleResponseCacheIdentity,
  PUBLIC_CONTENT_FENCE_PREFIX
} from "./shared.js"

/**
 * @typedef {object} FenceStore
 * @property {(key: string) => Promise<unknown>} get
 * @property {(keys: string[]) => Promise<unknown[]>} [mget]
 */
/**
 * @typedef {object} ArticleResponseFencePair
 * @property {string} publicContentFence
 * @property {string} permissionFence
 */
/**
 * @typedef {object} PermissionFenceVersions
 * @property {string} sitePermissionFence
 * @property {string} userPermissionFence
 */

/**
 * @param {unknown} value
 * @returns {string | null}
 */
export const normalizeFenceVersion = (value) => {
  if (value === undefined || value === null) return "0"
  if (typeof value !== "string" || !/^\d+$/.test(value)) return null
  return value
}

/**
 * @param {unknown} value
 * @returns {PermissionFenceVersions | null}
 */
export const parsePermissionFence = (value) => {
  if (typeof value !== "string") return null
  const match = /^site=(\d+),user=(\d+)$/.exec(value)
  if (!match) return null

  return {
    sitePermissionFence: match[1],
    userPermissionFence: match[2]
  }
}

/** @param {number} siteId */
export const buildPublicContentFenceKey = (siteId) => {
  return `${PUBLIC_CONTENT_FENCE_PREFIX}:${siteId}:version`
}

/** @param {number} siteId */
export const buildAnonymousPermissionFenceKeys = (siteId) => {
  return {
    siteKey: `permission:site:${siteId}:version`,
    userKey: `permission:site:${siteId}:user:anonymous:version`
  }
}

/**
 * @param {{
 *   siteId: number
 *   siteSlug: string
 *   requestHost: string
 *   route?: { slug?: string; extra?: string } | null
 *   requestLocales: string[]
 *   backendLocales: string[]
 *   publicContentFence: string
 *   permissionFence: string
 * }} input
 */
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

/**
 * @param {{ store?: FenceStore | null; siteId: number }} input
 * @returns {Promise<ArticleResponseFencePair | null>}
 */
export const readAnonymousArticleResponseCacheFences = async ({ store, siteId }) => {
  if (!store || !Number.isInteger(siteId) || siteId <= 0) return null

  try {
    const publicContentFenceKey = buildPublicContentFenceKey(siteId)
    const { siteKey, userKey } = buildAnonymousPermissionFenceKeys(siteId)
    const values =
      typeof store.mget === "function"
        ? await store.mget([publicContentFenceKey, siteKey, userKey])
        : [
            await store.get(publicContentFenceKey),
            await store.get(siteKey),
            await store.get(userKey)
          ]
    const publicContentFence = normalizeFenceVersion(values[0])
    const sitePermissionFence = normalizeFenceVersion(values[1])
    const userPermissionFence = normalizeFenceVersion(values[2])

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
