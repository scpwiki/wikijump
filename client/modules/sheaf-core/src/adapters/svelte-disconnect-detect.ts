/**
 * Custom element that is used to detect when Svelte components are unmounted.
 */
export class DisconnectElement extends HTMLElement {
  /** Element/tag name that this element is registered with. */
  static tag = "svelte-cm-disconnect-container"

  disconnectedCallback() {
    this.dispatchEvent(new CustomEvent("disconnected"))
  }
}

if (!customElements.get(DisconnectElement.tag)) {
  customElements.define(DisconnectElement.tag, DisconnectElement)
}
