export const DEEPWELL_PERMISSION_DENIED = 3106

/**
 * Remove structured Deepwell diagnostics before a JSON-RPC error crosses
 * the Framerail public boundary. Plain string data is retained for
 * deliberately user-facing JSON-RPC errors.
 *
 * @template T
 * @param {T} response
 * @returns {T}
 */
export const stripPrivateDeepwellErrorData = (response) => {
  if (Array.isArray(response)) {
    return /** @type {T} */ (response.map(stripPrivateDeepwellErrorData))
  }

  if (response === null || typeof response !== "object") return response

  const error = /** @type {{ error?: unknown }} */ (response).error
  if (error === null || typeof error !== "object" || Array.isArray(error)) {
    return response
  }

  const errorObject = /** @type {Record<string, unknown>} */ (error)
  if (!("data" in errorObject) || typeof errorObject.data === "string") {
    return response
  }

  const publicError = { ...errorObject }
  delete publicError.data
  return /** @type {T} */ ({ ...response, error: publicError })
}
