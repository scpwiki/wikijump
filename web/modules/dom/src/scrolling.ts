export type ScrollDirection = "vertical" | "horizontal" | "both"

/**
 * Checks if an element scrolls vertically. Only checks if the element _is
 * scrolling_, not if it _can scroll_.
 *
 * @param element - The element to check.
 */
export function scrollsVertically(element: HTMLElement) {
  // @ts-ignore - only works in Firefox
  if (element.scrollTopMax !== undefined) return element.scrollTopMax > 0

  // weird edge case, not sure what this means
  if (element.clientHeight === 0) return false

  // we can't just use `element.scrollHeight > element.clientHeight`, because
  // that will return true if a child also scrolls, so we need to check
  // the overflow properties instead
  if (element.scrollHeight > element.clientHeight) {
    const overflow = window.getComputedStyle(element).overflowY
    if (overflow === "scroll" || overflow === "auto") return true
  }

  return false
}

/**
 * Checks if an element scrolls horizontally. Only checks if the element
 * _is scrolling_, not if it _can scroll_.
 *
 * @param element - The element to check.
 */
export function scrollsHorizontally(element: HTMLElement) {
  // @ts-ignore - only works in Firefox
  if (element.scrollLeftMax !== undefined) return element.scrollLeftMax > 0

  // weird edge case, not sure what this means
  if (element.clientWidth === 0) return false

  // we can't just use `element.scrollWidth > element.clientWidth`, because
  // that will return true if a child also scrolls, so we need to check
  // the overflow properties instead
  if (element.scrollWidth > element.clientWidth) {
    const overflow = window.getComputedStyle(element).overflowX
    if (overflow === "scroll" || overflow === "auto") return true
  }

  return false
}

/**
 * Checks if an element scrolls. Only checks if the element _is scrolling_,
 * not if it _can scroll_.
 *
 * @param element - The element to check.
 * @param dir - The direction to check. Defaults to `both`.
 */
export function scrolls(element: HTMLElement, dir: ScrollDirection = "both") {
  // prettier-ignore
  switch (dir) {
    case "both":       return scrollsVertically(element) || scrollsHorizontally(element)
    case "vertical":   return scrollsVertically(element)
    case "horizontal": return scrollsHorizontally(element)
  }
}

/**
 * Finds the first parent of the given element that is scrolling. If the
 * given element itself is scrolling, it will be returned. If no scrolling
 * element is found, `null` is returned.
 *
 * @param element - The element to find the scrolling element for.
 * @param dir - The direction to check for scrolling. Defaults to `"vertical"`.
 */
export function scrollElement(element: HTMLElement, dir: ScrollDirection = "vertical") {
  let node: HTMLElement | null = element
  while (node) {
    if (scrolls(node, dir)) return node
    node = node.parentElement
  }
  return null
}
