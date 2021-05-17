import { Media } from "./media"
import * as easings from "svelte/easing"

export interface AnimOpts {
  delay?: number
  duration?: number
  easing?: keyof typeof easings
  forceReduced?: boolean
  css: (t: number, u: number) => string | string[]
}

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
