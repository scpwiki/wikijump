export abstract class BaseButton extends HTMLElement {
  protected whenKeydown?(evt: KeyboardEvent): void
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

  get disabled() {
    return this.hasAttribute("disabled")
  }

  private baseEnsureAttributes() {
    if (!this.hasAttribute("tab-index")) this.setAttribute("tab-index", "0")
    if (!this.hasAttribute("role")) this.setAttribute("role", "button")
  }

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
