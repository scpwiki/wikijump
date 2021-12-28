export interface DisplayOpts {
  visible?: () => void
  hidden?: () => void
}

/**
 * Observer that watches for if an element is being displayed in the DOM.
 * This isn't about the viewport, it's only about the DOM. This state is
 * effected by the `display` CSS property, among other things.
 */
export class DisplayObserver {
  /** Options for the observer. */
  private declare opts: DisplayOpts

  /**
   * Internal `ResizeObserver` that fires a callback when the element
   * displayed state changes.
   */
  private declare observer: ResizeObserver

  /** The last state that was set. */
  private last: boolean | null = null

  /** The target being observed. */
  declare target: HTMLElement

  /**
   * @param target - The element to observe.
   * @param opts - Options for the observer.
   */
  constructor(target: HTMLElement, opts: DisplayOpts = {}) {
    this.target = target
    this.opts = opts
    this.observer = new ResizeObserver(this.handler.bind(this))
    this.observer.observe(target)

    // fixes edge case - 0 size element is already invisible
    if (!this.visible) this.handler()
  }

  /** True if the target element is visible. */
  get visible() {
    // per the spec, this value is `null` if the element is not being displayed
    return this.target.offsetParent !== null
  }

  /** Internal handler for the resize observer events. */
  private handler() {
    const state = this.visible

    if (state === this.last) return

    this.last = state

    if (state) this.opts.visible?.()
    else this.opts.hidden?.()

    // in order for this ResizeObserver trick to work,
    // the element being observed needs to have a minimum size.
    // so we'll set it to 1px when it's hidden, which will
    // trigger a resize event when the element is shown again.

    if (state) {
      this.target.style.minHeight = ""
      this.target.style.minWidth = ""
      this.opts.visible?.()
    } else {
      this.target.style.minHeight = "1px"
      this.target.style.minWidth = "1px"
      this.opts.hidden?.()
    }
  }

  /** Updates the options for the observer. */
  update(opts: DisplayOpts = {}) {
    this.opts = opts
  }

  /** Destroys the observer. */
  destroy() {
    this.observer.disconnect()
  }
}
