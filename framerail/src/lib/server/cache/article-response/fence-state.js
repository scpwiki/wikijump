import { parsePermissionFence } from "./fence-values.js"

/**
 * @typedef {object} FenceSiteState
 * @property {string} publicContentFence
 * @property {string} sitePermissionFence
 * @property {string} userPermissionFence
 */
/**
 * @typedef {object} FencePair
 * @property {string} publicContentFence
 * @property {string} permissionFence
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

/** @param {{ clearHotResponses: () => void }} input */
export const createArticleResponseFenceState = ({ clearHotResponses }) => {
  /** @type {Map<number, FenceSiteState>} */
  const sites = new Map()
  let trusted = false
  let fenceRevision = 0

  const poison = () => {
    fenceRevision += 1
    trusted = false
    sites.clear()
    clearHotResponses()
  }

  /**
   * @param {number} siteId
   * @returns {FencePair | null}
   */
  const readFences = (siteId) => {
    const site = sites.get(siteId)
    if (!site) return null
    return {
      publicContentFence: site.publicContentFence,
      permissionFence: `site=${site.sitePermissionFence},user=${site.userPermissionFence}`
    }
  }

  /**
   * @param {{
   *   siteId: number
   *   fences: FencePair | null
   *   seedRevision: number
   * }} input
   * @returns {FenceSiteState | null}
   */
  const seedSite = ({ siteId, fences, seedRevision }) => {
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

  /** @param {PublicContentFenceMessage} message */
  const applyPublicContentMessage = (message) => {
    fenceRevision += 1
    const site = sites.get(message.siteId)
    if (!site) {
      clearHotResponses()
      return
    }
    if (BigInt(message.version) <= BigInt(site.publicContentFence)) return
    site.publicContentFence = message.version
    clearHotResponses()
  }

  /** @param {AnonymousPermissionFenceMessage} message */
  const applyAnonymousPermissionMessage = (message) => {
    fenceRevision += 1
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

  return {
    isTrusted: () => trusted,
    revision: () => fenceRevision,
    markTrusted() {
      trusted = true
    },
    poison,
    seedSite,
    readFences,

    /**
     * @param {{
     *   siteId: number
     *   publicContentFence: string
     *   permissionFence: string
     * }} input
     */
    areFencesCurrent({ siteId, publicContentFence, permissionFence }) {
      if (!trusted) return null
      const current = readFences(siteId)
      if (!current) return null
      return (
        current.publicContentFence === publicContentFence &&
        current.permissionFence === permissionFence
      )
    },

    /** @param {number} siteId */
    canValidateFencesLocally(siteId) {
      return trusted && sites.has(siteId)
    },

    /** @param {FenceInvalidationMessage} message */
    applyMessage(message) {
      if (message.type === "public-content") {
        applyPublicContentMessage(message)
      } else if (message.type === "anonymous-permission") {
        applyAnonymousPermissionMessage(message)
      }
    }
  }
}
