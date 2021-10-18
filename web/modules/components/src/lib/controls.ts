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

export interface WhileHeldOpts {
  /** Fired when the node is first held down. */
  pressed?: (node: HTMLElement) => void
  /** Fired when the node is released. */
  released?: (node: HTMLElement) => void
}

/**
 * Svelte `use` compatible function for firing callbacks when an element is
 * held down.
 */
export function whileHeld<T extends HTMLElement>(node: T, opts: WhileHeldOpts) {
  let curOpts = opts
  let held = false

  const pressed = () => {
    if (held) return
    held = true
    if (curOpts.pressed) curOpts.pressed(node)
  }

  const released = () => {
    if (!held) return
    held = false
    if (curOpts.released) curOpts.released(node)
  }

  node.addEventListener("pointerdown", pressed)
  node.addEventListener("pointerup", released)
  node.addEventListener("pointercancel", released)

  return {
    update(newOpts: WhileHeldOpts) {
      curOpts = newOpts
    },

    destroy() {
      node.removeEventListener("pointerdown", pressed)
      node.removeEventListener("pointerup", released)
      node.removeEventListener("pointercancel", released)
    }
  }
}
