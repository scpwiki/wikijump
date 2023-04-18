// Wrapper around fetch() to provide timeouts.

export const DEFAULT_TIMEOUT = 2000

export function wjfetch(url, options = {}) {
  let timeout = DEFAULT_TIMEOUT
  if (options.timeout) {
    timeout = options.timeout
    delete options.timeout
  }

  return fetch(url, { signal: AbortSignal.timeout(timeout), ...options })
}
