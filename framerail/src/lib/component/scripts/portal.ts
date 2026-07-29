/**
 * A Svelte use action that will 'portal' to the given target and append
 * the element to that target. The target can either be a direct reference
 * to the element, or a query selector string.
 */
export function portal(elem: Element, target: string | Element) {
  const update = (target: string | Element) => {
    let targetElem: Element | null

    if (typeof target === "string") {
      targetElem = document.querySelector(target)
    } else {
      targetElem = target
    }

    if (targetElem) targetElem.appendChild(elem)
    else throw new Error("Invalid portal target!")
  }

  update(target)

  return {
    update,
    destroy() {
      if (elem.parentElement) elem.parentElement.removeChild(elem)
    }
  }
}
