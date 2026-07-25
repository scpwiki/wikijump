import { normalizeFenceVersion } from "./fence-values.js"

/**
 * @typedef {object} PublicContentFenceMessage
 * @property {"public-content"} type
 * @property {number} siteId
 * @property {string} version
 */
/**
 * @typedef {object} AnonymousPermissionFenceMessage
 * @property {"anonymous-permission"} type
 * @property {number} siteId
 * @property {string} siteVersion
 * @property {string} userVersion
 */
/**
 * @typedef {object} UserPermissionFenceMessage
 * @property {"user-permission"} type
 */
/**
 * @typedef {PublicContentFenceMessage
 *   | AnonymousPermissionFenceMessage
 *   | UserPermissionFenceMessage} FenceInvalidationMessage
 */

/**
 * @param {unknown} value
 * @returns {number | null}
 */
const messageSiteId = (value) => {
  return Number.isInteger(value) && Number(value) > 0 ? Number(value) : null
}

/**
 * @param {unknown} payload
 * @returns {FenceInvalidationMessage | null}
 */
export const parseFenceInvalidationMessage = (payload) => {
  if (typeof payload !== "string") return null

  let parsed
  try {
    parsed = JSON.parse(payload)
  } catch {
    return null
  }
  if (!parsed || typeof parsed !== "object") return null
  const message = /** @type {Record<string, unknown>} */ (parsed)
  const siteId = messageSiteId(message.site_id)

  if (message.type === "public-content") {
    const version = normalizeFenceVersion(message.version)
    return siteId && version !== null ? { type: "public-content", siteId, version } : null
  }

  if (message.type === "anonymous-permission") {
    const siteVersion = normalizeFenceVersion(message.site_version)
    const userVersion = normalizeFenceVersion(message.user_version)
    return siteId && siteVersion !== null && userVersion !== null
      ? { type: "anonymous-permission", siteId, siteVersion, userVersion }
      : null
  }

  if (message.type === "user-permission") {
    const version = normalizeFenceVersion(message.version)
    return siteId &&
      Number.isInteger(message.user_id) &&
      Number(message.user_id) > 0 &&
      version
      ? { type: "user-permission" }
      : null
  }

  return null
}
