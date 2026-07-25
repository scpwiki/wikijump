import { ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL } from "./shared.js"
import { parseFenceInvalidationMessage } from "./fence-message.js"
import { createArticleResponseFenceState } from "./fence-state.js"
import { readAnonymousArticleResponseCacheFences } from "./fence-values.js"

/**
 * @typedef {object} FenceStore
 * @property {(key: string) => Promise<unknown>} get
 * @property {(keys: string[]) => Promise<unknown[]>} [mget]
 */
/** @typedef {{ clear?: () => void }} HotResponseCache */
/** @typedef {{ close?: () => void }} FenceSubscription */
/**
 * @typedef {object} FenceSubscriber
 * @property {(callbacks: {
 *   channel: string
 *   onSubscribed: () => void
 *   onMessage: (payload: string) => void
 *   onDisconnect: () => void
 *   onMalformed: () => void
 * }) => FenceSubscription | null | undefined} subscribe
 */

export {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousPermissionFenceKeys,
  buildPublicContentFenceKey,
  readAnonymousArticleResponseCacheFences
} from "./fence-values.js"

/**
 * @param {{
 *   store?: FenceStore | null
 *   subscriber?: FenceSubscriber | null
 * }} [options]
 */
export const createMemoryArticleResponseFenceCache = ({ store, subscriber } = {}) => {
  /** @type {Set<HotResponseCache>} */
  const hotCaches = new Set()
  const clearHotResponses = () => {
    for (const hotCache of hotCaches) hotCache.clear?.()
  }
  const state = createArticleResponseFenceState({ clearHotResponses })

  /** @param {number} siteId */
  const seedSite = async (siteId) => {
    const seedRevision = state.revision()
    const fences = await readAnonymousArticleResponseCacheFences({ store, siteId })
    const seeded = state.seedSite({ siteId, fences, seedRevision })
    return seeded ? state.readFences(siteId) : null
  }

  /** @param {string} payload */
  const applyMessage = (payload) => {
    const message = parseFenceInvalidationMessage(payload)
    if (!message) {
      state.poison()
      return false
    }
    state.applyMessage(message)
    return true
  }

  /** @type {FenceSubscription | null} */
  let subscription = null
  const api = {
    /** @param {HotResponseCache | null | undefined} hotCache */
    attachHotCache(hotCache) {
      if (hotCache) hotCaches.add(hotCache)
    },

    /** @param {{ siteId: number }} input */
    async readFences({ siteId }) {
      if (!state.isTrusted()) {
        return readAnonymousArticleResponseCacheFences({ store, siteId })
      }
      return state.readFences(siteId) ?? (await seedSite(siteId))
    },

    /**
     * @param {{
     *   siteId: number
     *   publicContentFence: string
     *   permissionFence: string
     * }} input
     */
    areFencesCurrent(input) {
      return state.areFencesCurrent(input)
    },

    /** @param {{ siteId: number }} input */
    canValidateFencesLocally({ siteId }) {
      return state.canValidateFencesLocally(siteId)
    },

    markSubscribedForTest: async () => {
      state.markTrusted()
    },

    markDisconnectedForTest: () => {
      state.poison()
    },

    /** @param {string} payload */
    applyMessageForTest: async (payload) => {
      applyMessage(payload)
    },

    close() {
      subscription?.close?.()
    }
  }

  subscription =
    subscriber?.subscribe?.({
      channel: ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL,
      onSubscribed: () => state.markTrusted(),
      onMessage: applyMessage,
      onDisconnect: state.poison,
      onMalformed: state.poison
    }) ?? null

  return api
}
