/* eslint-disable @typescript-eslint/unbound-method */
export interface KeyHandler {
  /**
   * The key to listen to. Must be in an exact format.
   *
   * Supports some special values:
   *
   * - `left-click`: Left mouse click.
   * - `right-click`: Right mouse click.
   * - `click`: Listens for the `click` event.
   * - `auxclick`: Listens for the `auxclick` event.
   * - `dblclick`: Listens for the `dblclick` event.
   * - `contextmenu`: Listens for the `contextmenu` event.
   *
   * @see https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values
   */
  key: string
  /** If true, fires `evt.preventDefault()` for the keystroke. */
  preventDefault?: boolean
  /** Callback to fire. */
  do?: AnyFunction
}

export class KeyObserver {
  private declare target: HTMLElement
  private declare handlers: KeyHandler[]

  constructor(target: HTMLElement, handlers: Arrayable<KeyHandler>) {
    this.target = target

    if (!Array.isArray(handlers)) handlers = [handlers]
    this.handlers = handlers

    this.keyHandler = this.keyHandler.bind(this)
    this.clickHandler = this.clickHandler.bind(this)

    this.target.addEventListener("keydown", this.keyHandler)
    this.target.addEventListener("mousedown", this.clickHandler)
    this.target.addEventListener("click", this.clickHandler)
    this.target.addEventListener("auxclick", this.clickHandler)
    this.target.addEventListener("dblclick", this.clickHandler)
    this.target.addEventListener("contextmenu", this.clickHandler)
  }

  private keyHandler(evt: KeyboardEvent) {
    // run through each handler and check if the keystroke matches
    this.handlers.forEach(handler => {
      if (evt.key === handler.key) {
        if (handler.preventDefault) evt.preventDefault()
        if (handler.do) handler.do()
      }
    })
  }

  private clickHandler(evt: MouseEvent) {
    this.handlers.forEach(handler => {
      let fire = false

      // we don't want left-click or right-click to repeatedly fire
      // for other events, so this event is handled differently
      if (evt.type === "mousedown") {
        if (evt.button === 0 && handler.key === "left-click") fire = true
        if (evt.button === 2 && handler.key === "right-click") fire = true
      }
      // for everything else we can just check if the evt type matches
      else {
        fire = evt.type === handler.key
      }

      if (fire) {
        if (handler.preventDefault) evt.preventDefault()
        if (handler.do) handler.do()
      }
    })
  }

  update(handlers: Arrayable<KeyHandler>) {
    if (!Array.isArray(handlers)) handlers = [handlers]
    this.handlers = handlers
  }

  destroy() {
    this.target.removeEventListener("keydown", this.keyHandler)
    this.target.removeEventListener("mousedown", this.clickHandler)
    this.target.removeEventListener("click", this.clickHandler)
    this.target.removeEventListener("auxclick", this.clickHandler)
    this.target.removeEventListener("dblclick", this.clickHandler)
    this.target.removeEventListener("contextmenu", this.clickHandler)
  }
}
