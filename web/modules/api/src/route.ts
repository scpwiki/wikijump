// organize-imports-ignore

// @ts-ignore - untyped package for some reason
import ziggyRoute from "ziggy-js"

// weirdly this is the best way to get the types out of the ziggy-js module
// @ts-ignore
import type Router from "@root/web/vendor/tightenco/ziggy/src/js/Router.js"

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
