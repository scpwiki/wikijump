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
