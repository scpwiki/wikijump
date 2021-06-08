/**
 * Custom element that is used to detect when Svelte components should be
 * mounted or unmounted.
 */
export class LifecycleElement extends HTMLElement {
  /** Element/tag name that this element is registered with. */
  static tag = "svelte-cm-lifecycle"

  /**
   * @param _mount - Function to be called whenever this element is mounted.
   * @param _unmount - Function to be called whenever this element is unmounted.
   */
  constructor(
    public _mount?: (dom: LifecycleElement) => void,
    public _unmount?: (dom: LifecycleElement) => void
  ) {
    super()
  }

  connectedCallback() {
    if (this._mount) this._mount(this)
    this.dispatchEvent(new CustomEvent("connected"))
  }

  disconnectedCallback() {
    if (this._unmount) this._unmount(this)
    this.dispatchEvent(new CustomEvent("disconnected"))
  }
}

if (!customElements.get(LifecycleElement.tag)) {
  customElements.define(LifecycleElement.tag, LifecycleElement)
}
