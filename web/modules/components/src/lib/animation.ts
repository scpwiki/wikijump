import * as easings from "svelte/easing"
import { Media } from "./media"

export interface AnimOpts {
  /** Delay before the animation plays, in or out. */
  delay?: number
  /** Length of animation, in miliseconds. */
  duration?: number
  /** Easing algorithm to use. */
  easing?: keyof typeof easings
  /** Forces the animation to play even with `prefers-reduced-motion: reduce`. */
  forceReduced?: boolean
  /**
   * Function that returns the styling that should be applied for a given time.
   *
   * @example
   *
   * ```ts
   * // fading animation
   * const anim = { css: t => `opacity: ${t}` }
   * ```
   */
  css: (t: number, u: number) => string | string[]
}

/**
 * Svelte `transition`, `in`, or `out` function for creating CSS-based animations.
 *
 * @example
 *
 * ```svelte
 * <!-- Scales the element in and out -->
 * <div
 *   transition:anim={{
 *     duration: 250,
 *     css: t => `transform: scale(${t})`
 *   }}
 * />
 * ```
 */
export function anim(
  node: Element,
  { delay = 0, duration = 500, easing = "quintInOut", forceReduced, css }: AnimOpts
) {
  if (!forceReduced && Media.reducedMotion) return {}

  const cb = (t: number, u: number) => {
    const result = css(t, u)
    return typeof result === "string" ? result : result.join(";")
  }

  return {
    delay,
    duration,
    easing: easings[easing],
    css: cb
  }
}

/** Animation that "unfolds" (or folds) an element via its `max-height`. */
export function unfold(node: Element, opts: Omit<AnimOpts, "css">) {
  const height = node.getBoundingClientRect().height
  const css = (t: number) => `max-height: ${t * height}px`
  return anim(node, { ...opts, css })
}
