import { applyFenceInvalidationToSites } from "./fence-reducer.js"
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

    /**
     * @param {Parameters<
     *   typeof applyFenceInvalidationToSites
     * >[0]["message"]} message
     */
    applyMessage(message) {
      if (message.type !== "user-permission") fenceRevision += 1
      applyFenceInvalidationToSites({ sites, message, clearHotResponses })
    }
  }
}
