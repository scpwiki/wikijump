/* eslint-disable @typescript-eslint/unbound-method */
import { mod } from "@wikijump/util"

// https://zellwk.com/blog/keyboard-focusable-elements/
/**
 * Returns a list of elements that can be reasonably expected as
 * programatically focusable directly under the given element.
 *
 * @param elem - The element to get the focusable descendants of.
 * @param anyIndex - Whether to include elements with `tabindex` set to `-1`.
 */
export function getFoci(elem: Element, anyIndex = false) {
  const arr = Array.from(
    elem.querySelectorAll<HTMLElement>(
      "a, button, input, textarea, select, details, [tabindex]"
    )
  ).filter(el => !el.hasAttribute("disabled"))

  return anyIndex ? arr : arr.filter(el => el.getAttribute("tabindex") !== "-1")
}

export type FocusGroupDirection = "vertical" | "horizontal"

export class FocusGroup {
  static keys = ["ArrowDown", "ArrowLeft", "ArrowRight", "ArrowUp", "End", "Home"]

  constructor(public target: HTMLElement, public direction: FocusGroupDirection) {
    this.handler = this.handler.bind(this)
    this.target.addEventListener("keydown", this.handler)
  }

  private handler(evt: KeyboardEvent) {
    const key = evt.key

    if (!FocusGroup.keys.includes(key)) return

    const focus = document.activeElement as HTMLElement | null
    if (!focus) return

    const foci = getFoci(this.target, true)
    if (!foci.length) return

    // maps every element to its index
    const fociMap = new Map<HTMLElement, number>()
    foci.map((el, i) => fociMap.set(el, i))

    const didArrow =
      this.direction === "vertical"
        ? key === "ArrowDown" || key === "ArrowUp"
        : key === "ArrowRight" || key === "ArrowLeft"

    const arrowDir = key === "ArrowUp" || key === "ArrowLeft" ? -1 : 1

    // modulo of next position (even -1) and array length causes the value to "wrap"
    // JS handles negative modulo weirdly so we don't use the operator directly
    const len = foci.length
    if (didArrow && fociMap.has(focus)) {
      foci[mod(fociMap.get(focus)! + arrowDir, len)].focus()
    } else if (key === "Home") {
      foci[0].focus()
    } else if (key === "End") {
      foci[len - 1].focus()
    } else {
      return
    }

    // if we passed a key check we will be here
    evt.preventDefault()
  }

  update(direction: FocusGroupDirection) {
    this.direction = direction
  }

  destroy() {
    this.target.removeEventListener("keydown", this.handler)
  }
}

export interface FocusOpts {
  /** Callback fired when the element (or any of its children) are focused. */
  focus?: () => void
  /** Callback fired when the element and its children are no longer focused. */
  blur?: () => void
}

/**
 * Fires callbacks for `focusin` and `focusout` events, but treats the
 * entire element's tree as singularly focusable. That is, unfocusing a
 * child of the given element by focusing to another child of the element
 * will not fire any additional callbacks.
 */
export class FocusObserver {
  /** Internal focused state of the tree. */
  private focused = false

  /**
   * @param target - The element to observe.
   * @param opts - Options for the observer.
   */
  constructor(private target: HTMLElement, private opts: FocusOpts) {
    this.focusin = this.focusin.bind(this)
    this.focusout = this.focusout.bind(this)

    target.addEventListener("focusin", this.focusin)
    target.addEventListener("focusout", this.focusout)
  }

  private focusin(evt: FocusEvent) {
    if (this.focused && this.target.contains(evt.target as HTMLElement)) return
    this.focused = true
    if (this.opts.focus) this.opts.focus()
  }

  private focusout(evt: FocusEvent) {
    if (this.target.contains(evt.relatedTarget as HTMLElement)) return
    this.focused = false
    if (this.opts.blur) this.opts.blur()
  }

  /**
   * Updates the observer configuration.
   *
   * @param opts - New options for the observer.
   */
  update(opts: FocusOpts) {
    this.opts = opts
  }

  /** Destroys the observer. */
  destroy() {
    this.target.removeEventListener("focusin", this.focusin)
    this.target.removeEventListener("focusout", this.focusout)
  }
}
