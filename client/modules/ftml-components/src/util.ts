/**
 * Safely defines a `customElement`.
 *
 * @param tagName - The tag name to define.
 * @param element - The class of the custom element.
 * @param options - Options for the custom element definition.
 */
export function defineElement(
  tagName: string,
  element: CustomElementConstructor,
  options?: ElementDefinitionOptions
) {
  if (!customElements.get(tagName)) {
    customElements.define(tagName, element, options)
  }
}

export interface HoverOpts {
  alsoOnFocus?: boolean
  on?: () => void
  off?: () => void
  move?: () => void
}

export function hover(element: HTMLElement, opts: HoverOpts) {
  if (opts.on) element.addEventListener("pointerover", opts.on)
  if (opts.off) element.addEventListener("pointerout", opts.off)
  if (opts.move) element.addEventListener("pointermove", opts.move)
  if (opts.alsoOnFocus) {
    if (opts.on) element.addEventListener("focus", opts.on)
    if (opts.off) element.addEventListener("blur", opts.off)
  }
}
