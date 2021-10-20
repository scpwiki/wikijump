export interface HoverOpts {
  /** If true, the element will be "hovered" when focused, too. */
  alsoOnFocus?: boolean
  /** Callback fired when the element is hovered over. */
  on?: () => void
  /** Callback fired when the element is no longer being hovered over. */
  off?: () => void
  /** Callback fired whenever the pointer moves. */
  move?: () => void
}

const HoverMediaQuery = matchMedia("(any-hover: hover), (hover: hover)")

/**
 * Utility for quickly adding hover event listeners to an element.
 *
 * @param element - The element to add the listeners to.
 * @param opts - The options to use.
 */
export function hover(element: HTMLElement, opts: HoverOpts) {
  let hovered = false
  let focused = false

  element.addEventListener("pointerover", () => {
    if (!HoverMediaQuery.matches) return
    hovered = true
    if (opts.on && !focused) opts.on()
  })

  element.addEventListener("pointerout", () => {
    if (!hovered && !HoverMediaQuery.matches) return
    hovered = false
    if (opts.off && !focused) opts.off()
  })

  if (opts.alsoOnFocus) {
    element.addEventListener("focusin", evt => {
      if (focused && element.contains(evt.target as HTMLElement)) return
      focused = true
      if (opts.on && !hovered) opts.on()
    })

    element.addEventListener("focusout", evt => {
      if (element.contains(evt.relatedTarget as HTMLElement)) return
      focused = false
      if (opts.off && !hovered) opts.off()
    })
  }

  if (opts.move) element.addEventListener("pointermove", opts.move)
}
