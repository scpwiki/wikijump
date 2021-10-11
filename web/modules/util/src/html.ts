// so we can load this module in workers:
let domParser: DOMParser
try {
  domParser = new DOMParser()
} catch {}

/** Takes a string of HTML and creates a {@link DocumentFragment}. */
export function toFragment(html: string) {
  const parsed = domParser.parseFromString(html, "text/html")
  const fragment = document.createDocumentFragment()
  fragment.append(...Array.from(parsed.body.children))
  return fragment
}

/**
 * **DOES NOT ESCAPE INPUT**
 *
 * Template string tag that creates a {@link DocumentFragment}.
 */
export function html(strings: TemplateStringsArray, ...subs: (string | string[])[]) {
  const src = strings.raw.reduce((prev, cur, idx) => {
    let sub = subs[idx - 1]
    if (Array.isArray(sub)) sub = sub.join("")
    return prev + sub + cur
  })
  return toFragment(src)
}

/**
 * **DOES NOT ESCAPE INPUT**
 *
 * Template string tag for creating a CSS stylesheet.
 */
export function css(strings: TemplateStringsArray, ...subs: (string | string[])[]) {
  const src = strings.raw.reduce((prev, cur, idx) => {
    let sub = subs[idx - 1]
    if (Array.isArray(sub)) sub = sub.join("")
    return prev + sub + cur
  })
  const style = document.createElement("style")
  style.textContent = src
  return style
}

/** Autonomous custom element class type. */
export type CustomElement = typeof HTMLElement & { tag: string }

/**
 * Adds an autonomous custom element to the registry.
 *
 * If the `global` variable is given, you'll need to also add the element
 * to the `Window` interface:
 *
 * ```ts
 * declare global {
 *   interface Window {
 *     CustomElementName: typeof CustomElement
 *   }
 * }
 * ```
 *
 * @param element - The custom element to add.
 * @param global - Adds the element to the global scope with the given name.
 */
export function addElement<E extends CustomElement>(element: E, global?: string) {
  if (!customElements.get(element.tag)) customElements.define(element.tag, element)

  // adds the element to the global scope
  // this is apparently standard practice,
  // GitHub does this with all of their elements
  // @ts-ignore
  if (global) globalThis[global] = element
}

/**
 * Upgrades all autonomous custom elements of type `to` under the given
 * `root` element.
 *
 * @param root - The root element to search for autonomous custom elements.
 * @param to - The custom element type to upgrade.
 */
export function upgrade(root: HTMLElement, to: CustomElement) {
  const elements = root.querySelectorAll(to.tag)
  for (let i = 0; i < elements.length; i++) {
    if (!(elements[i] instanceof to)) customElements.upgrade(elements[i])
  }
}

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

/**
 * Utility for quickly adding hover event listeners to an element.
 *
 * @param element - The element to add the listeners to.
 * @param opts - The options to use.
 */
export function hover(element: HTMLElement, opts: HoverOpts) {
  if (opts.on) element.addEventListener("pointerover", opts.on)
  if (opts.off) element.addEventListener("pointerout", opts.off)
  if (opts.move) element.addEventListener("pointermove", opts.move)
  if (opts.alsoOnFocus) {
    if (opts.on) element.addEventListener("focus", opts.on)
    if (opts.off) element.addEventListener("blur", opts.off)
  }
}

const OBSERVER_CONFIG = {
  childList: true,
  subtree: true,
  attributes: true,
  characterData: true
}

/**
 * Starts observing a target element using a `MutationObserver`.
 *
 * @param target - The target element to observe.
 * @param callback - The callback to call when the target element changes.
 */
export function observe(
  target: HTMLElement,
  callback: (changes: MutationRecord[]) => void
) {
  const observer = new MutationObserver(callback)
  observer.observe(target, OBSERVER_CONFIG)
  return observer
}

/**
 * Decorator that pauses an element's `MutationObserver` during a method
 * call. The observer needs to be in a public property named `observer`.
 */
export function pauseObservation(
  target: HTMLElement & { observer: MutationObserver },
  _key: string,
  descriptor: PropertyDescriptor
) {
  const method = descriptor.value
  const async = method.constructor.name === "AsyncFunction"

  if (async) {
    descriptor.value = async function (this: typeof target, ...args: any[]) {
      this.observer.disconnect()
      const result = await method.apply(this, args)
      this.observer.observe(this, OBSERVER_CONFIG)
      return result
    }
  } else {
    descriptor.value = function (this: typeof target, ...args: any[]) {
      this.observer.disconnect()
      const result = method.apply(this, args)
      this.observer.observe(this, OBSERVER_CONFIG)
      return result
    }
  }
}
