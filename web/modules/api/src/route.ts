// apparently Ziggy isn't on NPM...
// @ts-ignore
import ziggyRoute from "@root/web/vendor/tightenco/ziggy/src/js/index.js"
import type Router from "@root/web/vendor/tightenco/ziggy/src/js/Router.js"

// Ziggy is actually typed, but its actual `route` function isn't.

/** Gets the route state. */
export function route(): Router
/**
 * Gets route's URL via its name.
 *
 * @param name - The route's name.
 * @param params - The route's parameters, if any.
 * @param absolute - If true, the route will be given as a full URL.
 */
export function route(
  name: string,
  params?: Record<string, number | string>,
  absolute?: boolean
): string
export function route(
  name?: string,
  params?: Record<string, number | string>,
  absolute = false
) {
  return ziggyRoute(name, params, absolute)
}
