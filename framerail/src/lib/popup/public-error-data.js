/**
 * Return only deliberately public, plain-string error detail for the
 * popup. Structured diagnostics are never rendered.
 *
 * @param {unknown} data
 * @returns {string | null}
 */
export const publicErrorExtraMessage = (data) =>
  typeof data === "string" && data.length > 0 ? data : null
