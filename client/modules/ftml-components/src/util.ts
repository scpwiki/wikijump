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
