/**
 * @file Actions for Svelte components.
 */

/**
 * Inserts an element into the current element.
 *
 * @param element - The current element.
 * @param childElement - The element to append.
 */
export const contain: SvelteAction = (element, childElement: HTMLElement) => {
  element.appendChild(childElement)
}
