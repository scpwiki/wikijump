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

const OBSERVER_CONFIG = {
  childList: true,
  subtree: true,
  attributes: true,
  characterData: true
}

export function observe(
  target: HTMLElement,
  callback: (changes: MutationRecord[]) => void
) {
  const observer = new MutationObserver(callback)
  observer.observe(target, OBSERVER_CONFIG)
  return observer
}

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
