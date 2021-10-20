/* eslint-disable @typescript-eslint/unbound-method */
export interface WhileHeldOpts {
  /** Fired when the node is first held down. */
  pressed?: (node: HTMLElement) => void
  /** Fired when the node is released. */
  released?: (node: HTMLElement) => void
}

/** Observer for firing callbacks while an element is held down. */
export class HeldObserver {
  /** Current held state. */
  private held = false

  /**
   * @param target - The element to observe.
   * @param opts - Options for the observer.
   */
  constructor(private target: HTMLElement, private opts: WhileHeldOpts) {
    this.pressed = this.pressed.bind(this)
    this.released = this.released.bind(this)

    target.addEventListener("pointerdown", this.pressed)
    target.addEventListener("pointerup", this.released)
    target.addEventListener("pointercancel", this.released)
  }

  /** Fired when the element is pressed. */
  private pressed() {
    if (this.held) return
    this.held = true
    if (this.opts.pressed) this.opts.pressed(this.target)
  }

  /** Fired when the element is released. */
  private released() {
    if (!this.held) return
    this.held = false
    if (this.opts.released) this.opts.released(this.target)
  }

  /**
   * Updates the current observer configuration.
   *
   * @param opts - New options for the observer.
   */
  update(opts: WhileHeldOpts) {
    this.opts = opts
  }

  /** Destroys the observer. */
  destroy() {
    this.target.removeEventListener("pointerdown", this.pressed)
    this.target.removeEventListener("pointerup", this.released)
    this.target.removeEventListener("pointercancel", this.released)
  }
}
