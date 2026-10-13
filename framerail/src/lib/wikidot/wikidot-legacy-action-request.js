/**
 * @typedef {{
 *   fetch(
 *     input: string,
 *     init?: RequestInit
 *   ): Promise<{ text(): Promise<string> }>
 *   deserialize(value: string): {
 *     type: string
 *     data?: { res?: unknown; message?: string }
 *   }
 * }} LegacyRequestRuntime
 */

/**
 * @param {LegacyRequestRuntime} runtime
 * @param {string} action
 * @param {unknown} [body]
 */
const submit = async (runtime, action, body) => {
  const response = await runtime.fetch(`?/${action}`, {
    method: "POST",
    credentials: "same-origin",
    ...(body === undefined ? {} : { body: JSON.stringify(body) })
  })
  const result = runtime.deserialize(await response.text())
  if (result.type === "failure") {
    throw new Error(result.data?.message ?? "Legacy page action failed.")
  }
  return result.data?.res
}

/**
 * Submit a set-tags descriptor by server-issued ordinal, never by client
 * tags.
 *
 * @param {LegacyRequestRuntime} runtime
 * @param {{
 *   pageId: number
 *   lastRevisionId: number
 *   actionIndex: number
 *   actionFingerprint: string
 * }} input
 */
export const requestLegacySetTags = (runtime, input) => {
  return submit(runtime, "legacySetTags", {
    actionIndex: input.actionIndex,
    actionFingerprint: input.actionFingerprint,
    lastRevisionId: input.lastRevisionId,
    pageId: input.pageId
  })
}

/**
 * Submit one server-issued Rate action without a client-selected vote
 * value.
 *
 * @param {LegacyRequestRuntime} runtime
 * @param {{
 *   pageId: number
 *   lastRevisionId: number
 *   actionIndex: number
 *   actionFingerprint: string
 * }} input
 */
export const requestLegacyRate = (runtime, input) => {
  return /** @type {Promise<{ score: number } | undefined>} */ (
    submit(runtime, "legacyRate", {
      actionFingerprint: input.actionFingerprint,
      actionIndex: input.actionIndex,
      lastRevisionId: input.lastRevisionId,
      pageId: input.pageId
    })
  )
}

/**
 * Submit one server-issued Rate cancellation descriptor.
 *
 * @param {LegacyRequestRuntime} runtime
 * @param {{
 *   pageId: number
 *   lastRevisionId: number
 *   actionIndex: number
 *   actionFingerprint: string
 * }} input
 */
export const requestLegacyRateCancel = (runtime, input) => {
  return requestLegacyRate(runtime, input)
}

/**
 * Read the authoritative score for the route page after a vote mutation.
 *
 * The score request carries an explicit (empty) JSON body. SvelteKit rejects
 * a bodyless named-action POST with HTTP 415 before the action executes
 * (issue #2060), so this must never regress to a bodyless fetch.
 */
/**
 * @param {LegacyRequestRuntime} runtime
 * @returns {Promise<{ score: number } | undefined>}
 */
export const requestLegacyScore = (runtime) => {
  return /** @type {Promise<{ score: number } | undefined>} */ (
    submit(runtime, "score", {})
  )
}
