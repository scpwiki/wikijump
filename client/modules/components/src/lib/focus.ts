import { mod } from "wj-util"

// https://zellwk.com/blog/keyboard-focusable-elements/
/**
 * Returns a list of elements that can be reasonably expected
 * as programatically focusable directly under the given element.
 *
 * @param elem - The element to get the focusable descendants of.
 */
export function getFoci(elem: Element) {
  return Array.from(
    elem.querySelectorAll<HTMLElement>(
      "a, button, input, textarea, select, details, [tabindex]"
    )
  )
    .filter(el => !el.hasAttribute("disabled"))
    .filter(el => el.getAttribute("tabindex") !== "-1")
}

/**
 * Svelte use function for automatically handling directional key focus movement.
 * All descendants that are focusable with a non-negative tabindex will be
 * cycled through with the arrow keys.
 *
 * @param dir - Determines which pair of arrow keys to use.
 */
export function focusGroup(elem: Element, dir: "vertical" | "horizontal") {
  const keys = ["ArrowDown", "ArrowLeft", "ArrowRight", "ArrowUp", "End", "Home"]

  const handler = (evt: KeyboardEvent) => {
    if (!keys.includes(evt.key)) return

    const focus = document.activeElement as HTMLElement
    if (!focus) return

    const foci = getFoci(elem)
    if (!foci.length) return

    // maps every element to its index
    const fociMap = new Map<HTMLElement, number>()
    foci.map((el, i) => fociMap.set(el, i))

    const didArrow =
      (dir === "vertical" && (evt.key === "ArrowUp" || evt.key === "ArrowDown")) ||
      (dir === "horizontal" && (evt.key === "ArrowLeft" || evt.key === "ArrowRight"))

    const arrowDir = evt.key === "ArrowUp" || evt.key === "ArrowLeft" ? -1 : 1

    // modulo of next position (even -1) and array length causes the value to "wrap"
    // JS handles negative modulo weirdly so we don't use the operator directly
    const len = foci.length
    if (didArrow && fociMap.has(focus)) {
      foci[mod(fociMap.get(focus)! + arrowDir, len)].focus()
    } else if (evt.key === "Home") {
      foci[0].focus()
    } else if (evt.key === "End") {
      foci[len - 1].focus()
    } else {
      return
    }

    // if we passed a key check we will be here
    evt.preventDefault()
  }

  // typescript doesn't handle keydown correctly for some reason?
  elem.addEventListener("keydown", handler as any)

  return {
    update(newDir: "vertical" | "horizontal") {
      dir = newDir
    },
    destroy() {
      elem.removeEventListener("keydown", handler as any)
    }
  }
}
