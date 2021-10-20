import { FocusGroup, FocusGroupDirection } from "./focus"
import { HeldObserver, WhileHeldOpts } from "./key-handling"
import { SwipeObserver, SwipeOpts } from "./swipe"

export interface SvelteAction<T> {
  update: (value: T) => void
  destroy: () => void
}

/**
 * Starts an event listener that will recognize swipes on the specified
 * element. Works natively with Svelte elements, if used as an
 * `use:onSwipe` action. For basic usage, provide `direction` and
 * `callback` properties in the options object.
 *
 * @example
 *
 * ```svelte
 * <div use:onSwipe={{ callback: callback, direction: "up" }} />
 * ```
 *
 * @param target - The element to observe.
 * @param opts - The options to use.
 */
export function onSwipe(
  target: HTMLElement,
  opts: Partial<SwipeOpts>
): SvelteAction<SwipeOpts> {
  const swipe = new SwipeObserver(target, opts)
  return {
    update: (opts: Partial<SwipeOpts>) => swipe.update(opts),
    destroy: () => swipe.destroy()
  }
}

/**
 * Svelte `use` function for automatically handling directional key focus
 * movement. All descendants that are focusable with a non-negative
 * tabindex will be cycled through with the arrow keys.
 *
 * @param direction - Determines which pair of arrow keys to use.
 */
export function focusGroup(
  node: HTMLElement,
  direction: FocusGroupDirection
): SvelteAction<FocusGroupDirection> {
  const group = new FocusGroup(node, direction)
  return {
    update: (direction: FocusGroupDirection) => group.update(direction),
    destroy: () => group.destroy()
  }
}

/**
 * Svelte `use` compatible function for firing callbacks when an element is
 * held down.
 */
export function whileHeld(target: HTMLElement, opts: WhileHeldOpts) {
  const held = new HeldObserver(target, opts)
  return {
    update: (opts: WhileHeldOpts) => held.update(opts),
    destroy: () => held.destroy()
  }
}
