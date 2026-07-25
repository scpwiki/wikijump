/**
 * @typedef {object} FenceSiteState
 * @property {string} publicContentFence
 * @property {string} sitePermissionFence
 * @property {string} userPermissionFence
 */
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
/** @typedef {{ type: "user-permission" }} UserPermissionFenceMessage */
/**
 * @typedef {PublicContentFenceMessage
 *   | AnonymousPermissionFenceMessage
 *   | UserPermissionFenceMessage} FenceInvalidationMessage
 */

/**
 * @param {Map<number, FenceSiteState>} sites
 * @param {PublicContentFenceMessage} message
 * @param {() => void} clearHotResponses
 */
const applyPublicContentMessage = (sites, message, clearHotResponses) => {
  const site = sites.get(message.siteId)
  if (!site) {
    clearHotResponses()
    return
  }
  if (BigInt(message.version) <= BigInt(site.publicContentFence)) return
  site.publicContentFence = message.version
  clearHotResponses()
}

/**
 * @param {Map<number, FenceSiteState>} sites
 * @param {AnonymousPermissionFenceMessage} message
 * @param {() => void} clearHotResponses
 */
const applyAnonymousPermissionMessage = (sites, message, clearHotResponses) => {
  const site = sites.get(message.siteId)
  if (!site) {
    clearHotResponses()
    return
  }
  const siteAdvanced = BigInt(message.siteVersion) > BigInt(site.sitePermissionFence)
  const userAdvanced = BigInt(message.userVersion) > BigInt(site.userPermissionFence)
  if (!siteAdvanced && !userAdvanced) return
  site.sitePermissionFence = message.siteVersion
  site.userPermissionFence = message.userVersion
  clearHotResponses()
}

/**
 * @param {{
 *   sites: Map<number, FenceSiteState>
 *   message: FenceInvalidationMessage
 *   clearHotResponses: () => void
 * }} input
 */
export const applyFenceInvalidationToSites = ({ sites, message, clearHotResponses }) => {
  if (message.type === "public-content") {
    applyPublicContentMessage(sites, message, clearHotResponses)
  } else if (message.type === "anonymous-permission") {
    applyAnonymousPermissionMessage(sites, message, clearHotResponses)
  }
}
