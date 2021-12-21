import {
  FocusGroup,
  FocusObserver,
  type FocusGroupDirection,
  type FocusOpts
} from "./focus"
import { HeldObserver, type WhileHeldOpts } from "./held"
import { HoverObserver, type HoverOpts } from "./hover"
import { KeyObserver, type KeyHandler } from "./key-handling"
import { SwipeObserver, type SwipeOpts } from "./swipe"

/**
 * Starts an event listener that will recognize swipes on the specified
 * element. Works natively with Svelte elements, if used as an
 * `use:onSwipe` action. For basic usage, provide `direction` and
 * `callback` properties in the options object.
 *
 * @example
 *
 * ```svelte
 * ;<div use:onSwipe={{ callback: callback, direction: "up" }} />
 * ```
 */
export function onSwipe(target: HTMLElement, opts: Partial<SwipeOpts>) {
  const observer = new SwipeObserver(target, opts)
  return {
    update: (opts: Partial<SwipeOpts>) => observer.update(opts),
    destroy: () => observer.destroy()
  }
}

/**
 * Svelte `use` function for automatically handling directional key focus
 * movement. All descendants that are focusable with a non-negative
 * tabindex will be cycled through with the arrow keys.
 */
export function focusGroup(node: HTMLElement, direction: FocusGroupDirection) {
  const observer = new FocusGroup(node, direction)
  return {
    update: (direction: FocusGroupDirection) => observer.update(direction),
    destroy: () => observer.destroy()
  }
}

/** Svelte `use` function for handling keypresses. */
export function keyHandle(target: HTMLElement, handlers: Arrayable<KeyHandler>) {
  const observer = new KeyObserver(target, handlers)
  return {
    update: (handlers: Arrayable<KeyHandler>) => observer.update(handlers),
    destroy: () => observer.destroy()
  }
}

/** Svelte `use` function for firing callbacks when an element is held down. */
export function whileHeld(target: HTMLElement, opts: WhileHeldOpts) {
  const observer = new HeldObserver(target, opts)
  return {
    update: (opts: WhileHeldOpts) => observer.update(opts),
    destroy: () => observer.destroy()
  }
}

/**
 * Svelte `use` function for firing callbacks during hover (and optionally
 * focus) events.
 */
export function onHover(target: HTMLElement, opts: HoverOpts) {
  const observer = new HoverObserver(target, opts)
  return {
    update: (opts: HoverOpts) => observer.update(opts),
    destroy: () => observer.destroy()
  }
}

/** Svelte `use` function for firing callbacks for tree focus/blur events. */
export function onFocus(target: HTMLElement, opts: FocusOpts) {
  const observer = new FocusObserver(target, opts)
  return {
    update: (opts: FocusOpts) => observer.update(opts),
    destroy: () => observer.destroy()
  }
}
