import { route } from "@wikijump/api"

/**
 * Returns a dashboard route.
 *
 * @param path - The subpath.
 * @param params - The route parameters, if any.
 */
export function dashboardRoute(path = "", params: Record<string, string | number> = {}) {
  if (path.startsWith("/")) path = path.substring(1)
  return route("dashboard", { path, ...params })
}
