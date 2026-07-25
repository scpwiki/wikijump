// Wrapper around fetch() to provide timeouts.

export const DEFAULT_TIMEOUT = 1500

type WjFetchOptions = RequestInit & { timeout?: number }

export function wjfetch(url: RequestInfo | URL, options: WjFetchOptions = {}) {
  const { timeout = DEFAULT_TIMEOUT, ...fetchOptions } = options

  return fetch(url, { signal: AbortSignal.timeout(timeout), ...fetchOptions })
}
