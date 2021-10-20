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

export interface OnFocusDeepOpts {
  /** Callback fired when the element (or any of its children) are focused. */
  focus?: () => void
  /** Callback fired when the element and its children are no longer focused. */
  blur?: () => void
}

/**
 * Fires callbacks for `focusin` and `focusout` events, but treats the
 * entire element's tree as singularly focusable. That is, unfocusing a
 * child of the given element by focusing to another child of the element
 * will not fire any additional callbacks.
 *
 * @param element - The element to add the listeners to.
 * @param opts - The options to use.
 */
export function onFocusDeep(element: HTMLElement, opts: OnFocusDeepOpts) {
  let focused = false

  element.addEventListener("focusin", evt => {
    if (focused && element.contains(evt.target as HTMLElement)) return
    focused = true
    if (opts.focus) opts.focus()
  })

  element.addEventListener("focusout", evt => {
    if (element.contains(evt.relatedTarget as HTMLElement)) return
    focused = false
    if (opts.blur) opts.blur()
  })
}

export type ScrollDirection = "vertical" | "horizontal" | "both"

/**
 * Checks if an element scrolls vertically. Only checks if the element *is
 * scrolling*, not if it *can scroll*.
 *
 * @param element - The element to check.
 */
export function scrollsVertically(element: HTMLElement) {
  // @ts-ignore - only works in Firefox
  if (element.scrollTopMax !== undefined) return element.scrollTopMax > 0

  // weird edge case, not sure what this means
  if (element.clientHeight === 0) return false

  // we can't just use `element.scrollHeight > element.clientHeight`, because
  // that will return true if a child also scrolls, so we need to check
  // the overflow properties instead
  if (element.scrollHeight > element.clientHeight) {
    const overflow = window.getComputedStyle(element).overflowY
    if (overflow === "scroll" || overflow === "auto") return true
  }

  return false
}

/**
 * Checks if an element scrolls horizontally. Only checks if the element
 * *is scrolling*, not if it *can scroll*.
 *
 * @param element - The element to check.
 */
export function scrollsHorizontally(element: HTMLElement) {
  // @ts-ignore - only works in Firefox
  if (element.scrollLeftMax !== undefined) return element.scrollLeftMax > 0

  // weird edge case, not sure what this means
  if (element.clientWidth === 0) return false

  // we can't just use `element.scrollWidth > element.clientWidth`, because
  // that will return true if a child also scrolls, so we need to check
  // the overflow properties instead
  if (element.scrollWidth > element.clientWidth) {
    const overflow = window.getComputedStyle(element).overflowX
    if (overflow === "scroll" || overflow === "auto") return true
  }

  return false
}

/**
 * Checks if an element scrolls. Only checks if the element *is scrolling*,
 * not if it *can scroll*.
 *
 * @param element - The element to check.
 * @param dir - The direction to check. Defaults to `both`.
 */
export function scrolls(element: HTMLElement, dir: ScrollDirection = "both") {
  // prettier-ignore
  switch (dir) {
    case "both":       return scrollsVertically(element) || scrollsHorizontally(element)
    case "vertical":   return scrollsVertically(element)
    case "horizontal": return scrollsHorizontally(element)
  }
}

/**
 * Finds the first parent of the given element that is scrolling. If the
 * given element itself is scrolling, it will be returned. If no scrolling
 * element is found, `null` is returned.
 *
 * @param element - The element to find the scrolling element for.
 * @param dir - The direction to check for scrolling. Defaults to `"vertical"`.
 */
export function scrollElement(element: HTMLElement, dir: ScrollDirection = "vertical") {
  let node: HTMLElement | null = element
  while (node) {
    if (scrolls(node, dir)) return node
    node = node.parentElement
  }
  return null
}

/**
 * Checks if all of the given form inputs are valid via ordinary HTML validation.
 *
 * @param inputs - The form inputs to check. Can accept `undefined` or
 *   `null` values, which will count as "invalid" inputs.
 */
export function inputsValid(...inputs: (HTMLInputElement | undefined | null)[]) {
  for (const input of inputs) {
    if (
      !input ||
      !input.validity.valid ||
      input.value.length === 0 ||
      input.disabled ||
      input.readOnly
    ) {
      return false
    }
  }

  return true
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
