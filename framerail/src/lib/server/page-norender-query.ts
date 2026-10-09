// Query-string form of the Wikidot no-render selector.
//
// `/slug/norender/true` and `/slug?norender=true` are equivalent viewer
// requests. The query values observed to activate no-render are `true`, `t`,
// `1`, and `false`; `0`, an empty value, and a bare `?norender` leave the page
// rendered. Only that set is recognized, so no general boolean coercion
// happens here. A recognized value is forwarded to the view as the canonical
// path argument `norender/true`.

const NORENDER_KEY = "norender"
const ACTIVATING_VALUES: ReadonlySet<string> = new Set(["true", "t", "1", "false"])

export function queryNoRenderSelected(search: string): boolean {
  const params = new URLSearchParams(search)
  for (const [key, value] of params) {
    if (key.toLowerCase() !== NORENDER_KEY) continue
    // Only the first occurrence is considered.
    return ACTIVATING_VALUES.has(value)
  }
  return false
}

/**
 * Returns the article route with the no-render selector appended as a path
 * argument. Routes without a selector are returned unchanged, so their
 * view requests are identical to before.
 */
export function articleRouteWithQueryNoRender<T extends { extra?: string | null }>(
  route: T | null,
  search: string
): T | null {
  if (!route || !queryNoRenderSelected(search)) return route
  const extra = route.extra ?? ""
  const selector = `${NORENDER_KEY}/true`
  return { ...route, extra: extra === "" ? selector : `${extra}/${selector}` }
}
