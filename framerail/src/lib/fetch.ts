// Wrapper around fetch() to provide timeouts.

export const DEFAULT_TIMEOUT = 1500

export function wjfetch(url, options = {}) {
  const { timeout = DEFAULT_TIMEOUT, ...fetchOptions } = options

  return fetch(url, { signal: AbortSignal.timeout(timeout), ...fetchOptions })
}
