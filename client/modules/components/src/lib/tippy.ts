import tippy, { Props, roundArrow } from "tippy.js"
// Import CSS dynamically
import "tippy.js/animations/scale.css"
import "tippy.js/dist/svg-arrow.css"
import "tippy.js/dist/tippy.css"

const DEFAULT_TIPPY_OPTS: Partial<Props> = {
  ignoreAttributes: true,
  theme: "wikijump",
  arrow: roundArrow,
  animation: "scale",
  touch: ["hold", 600],
  duration: [50, 100],
  delay: [400, 50]
}

function parseTipOpts(elem: Element, opts: Partial<Props> | string) {
  if (opts) {
    // use:tippy="foo"
    if (typeof opts === "string") {
      opts = { content: opts }
      // use:tippy={{ opt: "bar" }} aria-label="foo"
    } else if (!opts.content) {
      opts.content = elem.getAttribute("aria-label") ?? ""
    }
    // use:tippy
  } else {
    opts = { content: elem.getAttribute("aria-label") ?? "" }
  }

  return { ...DEFAULT_TIPPY_OPTS, ...opts }
}

/**
 * Creates a Tippy.js tooltip instance for the element.
 *
 * The tooltip will derive its message from this list of sources, in order:
 *
 * - The value provided directly, if any
 * - The element's `aria-label` attribute, if it exists
 */
export function tip(elem: Element, opts: Partial<Props> | string = "") {
  opts = parseTipOpts(elem, opts)
  const tp = tippy(elem, opts)
  const setState = (content: unknown) => {
    if (!content) tp.disable()
    else tp.enable()
  }
  setState(opts.content)
  return {
    update(opts: Partial<Props> | string = "") {
      opts = parseTipOpts(elem, opts)
      tp.setProps(opts)
      setState(opts.content)
    },
    destroy() {
      tp.destroy()
    }
  }
}
