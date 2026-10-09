// Query-string form of the Wikidot no-render selector.
//
// `/slug/norender/true` and `/slug?norender=true` are equivalent viewer
// requests. Deepwell's PageOptions parser owns the value semantics (`true`,
// `t`, `1` activate; `false`, `0` do not), so the query selector is forwarded
// to the view as the same path argument it would be in the URL path.

const NORENDER_KEY = "norender"

// Only the characters a path argument can carry without becoming a second
// segment. Anything else is ignored rather than widened into a path.
const SAFE_NORENDER_VALUE = /^[A-Za-z0-9_.-]+$/u

export function queryNoRenderValue(search: string): string | null {
  const params = new URLSearchParams(search)
  for (const [key, value] of params) {
    if (key.toLowerCase() !== NORENDER_KEY) continue
    // A bare `?norender` does not select no-render, and only the first
    // occurrence is considered.
    if (value === "" || !SAFE_NORENDER_VALUE.test(value)) return null
    return value
  }
  return null
}

/**
 * Returns the article route with the query selector appended as a path
 * argument. Routes without a selector are returned unchanged, so their
 * view requests are identical to before.
 */
export function articleRouteWithQueryNoRender<T extends { extra?: string | null }>(
  route: T | null,
  search: string
): T | null {
  const value = queryNoRenderValue(search)
  if (!route || value === null) return route
  const extra = route.extra ?? ""
  const selector = `${NORENDER_KEY}/${value}`
  return { ...route, extra: extra === "" ? selector : `${extra}/${selector}` }
}
