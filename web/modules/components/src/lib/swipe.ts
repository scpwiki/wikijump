import { Direction, Gesture, gestureObserve } from "./gesture"

export interface OnSwipeOpts {
  /**
   * Can be used to enable and disable the swipe. Return true to enable
   * recognition, return false to disable recognition.
   */
  condition?: () => boolean
  /** Function to call upon the user swiping. */
  callback: (target: HTMLElement, gesture: Gesture) => void
  /**
   * Function that, if provided, is called on every event and given the
   * current swipe state.
   */
  eventCallback?: (target: HTMLElement, gesture: Gesture) => void
  /** Swipe direction to recognize. */
  direction: Direction | Direction[]
  /** Minimum distance in pixels needed for a swipe to count. */
  threshold: number
  /**
   * Minimum distance needed for the swipe recognition to be started. This
   * is so a user's pointer can placed down, be still, and then finally
   * swipe and still have it recognized.
   */
  minThreshold: number
  /**
   * If true, the swipe will be recognized even if the user hasn't lifted
   * their pointer yet.
   */
  immediate?: boolean
  /**
   * Duration of time (in miliseconds) after movement has begun that a
   * swipe will be recognized. Pass `0` or `false` to have no timeout.
   */
  timeout?: number | false
}

const ONSWIPE_DEFAULT_OPTS: OnSwipeOpts = {
  callback: () => null,
  direction: "up",
  immediate: true,
  threshold: 35,
  minThreshold: 10,
  timeout: 250
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
export function onSwipe(target: HTMLElement, opts: Partial<OnSwipeOpts>) {
  opts = { ...ONSWIPE_DEFAULT_OPTS, ...opts }

  const directions =
    typeof opts.direction === "string" ? [opts.direction] : opts.direction!

  let timeout: number | undefined
  let started = false
  let cancelled = false

  const cancel = (gesture: Gesture) => {
    cancelled = true
    if (started) opts.eventCallback?.(target, { ...gesture, type: "cancel" })
  }

  const handler = (gesture: Gesture) => {
    if (gesture.type === "start") {
      started = false
      cancelled = false
    }

    if (cancelled) return
    if (!started && opts.condition && !opts.condition()) return

    const { direction, dist, type } = gesture
    const validDirection = directions.includes(direction)
    const overThreshold = validDirection && dist > opts.threshold!
    const overMinimumThreshold = started || (validDirection && dist > opts.minThreshold!)

    // gesture is over threshold, but in the wrong direction
    if (!validDirection && dist > opts.minThreshold!) cancel(gesture)

    if (!cancelled) {
      // gesture started, start executing event callback
      if (overMinimumThreshold) {
        started = true
        opts.eventCallback?.(target, gesture)
      }
      // execute callback if valid && immediate mode or if the gesture ended
      if (overThreshold && ((type === "move" && opts.immediate) || type === "end")) {
        opts.callback!(target, gesture)
        cancelled = true
      }
      // ending the gesture and our timeout is running, cancel it
      else if (timeout && (type === "end" || type === "cancel")) {
        clearTimeout(timeout)
      }
      // gesture is running, but our timeout isn't running, start it
      else if (overMinimumThreshold && opts.timeout && !timeout) {
        setTimeout(() => cancel(gesture), opts.timeout)
      }
    }
  }

  const destroy = gestureObserve(target, handler)

  return {
    update(newOpts: Partial<OnSwipeOpts>) {
      opts = { ...ONSWIPE_DEFAULT_OPTS, ...newOpts }
    },

    destroy
  }
}
