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

/** Utility Svelte use function to handle key press events for the element. */
export function keyHandle(elem: Element, handlers: KeyHandler[] | KeyHandler) {
  const handler = (evt: KeyboardEvent) => {
    // convert shorthand into an array
    if (!(handlers instanceof Array)) handlers = [handlers]
    // run through each handler and check if the keystroke matches
    handlers.forEach(handler => {
      if (evt.key === handler.key) {
        if (handler.preventDefault) evt.preventDefault()
        if (handler.do) handler.do()
      }
    })
  }

  // more keydown type wackyness
  elem.addEventListener("keydown", handler as any)

  return {
    update(newHandlers: KeyHandler[] | KeyHandler) {
      handlers = newHandlers
    },
    destroy() {
      elem.removeEventListener("keydown", handler as any)
    }
  }
}
