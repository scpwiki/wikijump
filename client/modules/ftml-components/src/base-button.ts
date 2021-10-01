/**
 * Abstract custom element that can serve as a replacement for the
 * `<button>` element. It calls two methods, if they exist: `whenClicked`,
 * and `whenKeydown`.
 */
export abstract class BaseButton extends HTMLElement {
  /** Fired on `keydown` events. */
  protected whenKeydown?(evt: KeyboardEvent): void

  /**
   * Fired if the button is activated, either through clicking, pressing
   * space, or pressing enter.
   */
  protected whenClicked?(evt: MouseEvent): void

  constructor() {
    super()

    this.addEventListener("keydown", evt => this.baseKeydownHandler(evt))
    this.addEventListener("click", evt => this.baseClickHandler(evt))

    // we'll avoid using the `observedAttributes` property, so that
    // subclasses don't have to worry about extending that attribute's value
    const observer = new MutationObserver(() => this.baseEnsureAttributes())
    observer.observe(this, { attributes: true })
  }

  /** True if the button element has the `disabled` attribute. */
  get disabled() {
    return this.hasAttribute("disabled")
  }

  /** Ensures that accessibility attributes are set. */
  private baseEnsureAttributes() {
    if (!this.hasAttribute("tab-index")) this.setAttribute("tab-index", "0")
    if (!this.hasAttribute("role")) this.setAttribute("role", "button")
  }

  /** Internal handler for keydown events. */
  private baseKeydownHandler(evt: KeyboardEvent) {
    if (!this.disabled) {
      // apparently, it's not "Space", it's " ". thank you javascript
      if (evt.key === "Enter" || evt.key === " ") {
        this.click()
        evt.preventDefault()
      } else if (this.whenKeydown) {
        this.whenKeydown(evt)
      }
    }
  }

  /** Internal handler for click events. */
  private baseClickHandler(evt: MouseEvent) {
    if (!this.disabled && this.whenClicked) {
      this.whenClicked(evt)
      evt.preventDefault()
    }
  }

  // -- LIFECYCLE

  connectedCallback() {
    this.baseEnsureAttributes()
  }
}
