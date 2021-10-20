/* eslint-disable @typescript-eslint/unbound-method */
export interface KeyHandler {
  /**
   * The key to listen to. Must be in an exact format.
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

    this.handler = this.handler.bind(this)

    target.addEventListener("keydown", this.handler)
  }

  private handler(evt: KeyboardEvent) {
    // run through each handler and check if the keystroke matches
    this.handlers.forEach(handler => {
      if (evt.key === handler.key) {
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
    this.target.removeEventListener("keydown", this.handler)
  }
}
