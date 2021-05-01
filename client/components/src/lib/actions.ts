/**
 * @file Actions for Svelte components.
 */

/**
 * Takes the given element and inserts it into the current element.
 *
 * @param childElement - The element to append.
 */
export const contain: SvelteAction = (element, childElement: HTMLElement) => {
  element.appendChild(childElement)
}
