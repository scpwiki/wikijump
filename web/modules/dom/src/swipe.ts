import { clearTimeout, Timeout, timeout } from "@wikijump/util"
import { Gesture, GestureObserver, type Direction } from "./gesture"

export interface SwipeOpts {
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

const SWIPE_DEFAULT_OPTS: SwipeOpts = {
  callback: () => null,
  direction: "up",
  immediate: true,
  threshold: 35,
  minThreshold: 10,
  timeout: 250
}

export class SwipeObserver {
  /** Swipe recognition configuration. */
  private declare opts: SwipeOpts

  /** List of valid swipe directions. */
  private declare directions: Direction[]

  /** Internal {@link GestureObserver} used to get gesture information. */
  private declare observer: GestureObserver

  /** The current {@link Gesture}. */
  private gesture: Gesture | null = null

  /** The currently running {@link Timeout}. */
  private timeout: Timeout | null = null

  /** True if the swipe recognition is running. */
  private running = false

  /** True if the current swipe gesture is being ignored. */
  private ended = false

  /** The element being tracked for gestures. */
  declare target: HTMLElement

  /**
   * @param target - The target element to track.
   * @param opts - Swipe recognition configuration.
   */
  constructor(target: HTMLElement, opts: Partial<SwipeOpts> = {}) {
    this.target = target
    this.opts = { ...SWIPE_DEFAULT_OPTS, ...opts }

    this.directions = Array.isArray(this.opts.direction)
      ? this.opts.direction
      : [this.opts.direction]

    this.observer = new GestureObserver(target, gesture => {
      this.gesture = gesture
      this.handler()
    })
  }

  /** Runs the configured condition, if present. Returns `true` if it isn't present. */
  private checkCondition() {
    if (!this.opts.condition) return true
    return this.opts.condition()
  }

  /** Fire the configured callback, if present. */
  private fireCallback() {
    if (this.gesture && this.opts.callback) {
      this.opts.callback(this.target, this.gesture)
    }
  }

  /** Fire the configured event callback ,if present. */
  private fireEventCallback() {
    if (this.gesture && this.opts.eventCallback) {
      this.opts.eventCallback(this.target, this.gesture)
    }
  }

  /** Reset internal state. */
  private reset() {
    clearTimeout(this.timeout)
    this.timeout = null
    this.ended = false
    this.running = false
  }

  /** Cancel the currently running swipe. */
  private cancel() {
    clearTimeout(this.timeout)
    this.ended = true
    if (this.running && this.gesture) {
      this.gesture = this.gesture.replace({ type: "cancel" })
      this.fireEventCallback()
    }
  }

  /** Handler that is called by the {@link observer}. */
  private handler() {
    const gst = this.gesture!

    if (gst.is("start")) this.reset()

    // ended or condition prevents us from continuing
    if (this.ended || (!this.running && !this.checkCondition())) return

    const valid = gst.same(this.directions)
    const overMinThreshold = gst.over(this.opts.minThreshold)

    // gesture is over minimum threshold, but in the wrong direction
    if (!valid && overMinThreshold) {
      this.cancel()
      return
    }

    if (valid && !this.running && overMinThreshold) this.running = true

    if (this.running) {
      const overThreshold = gst.over(this.opts.threshold)
      const canFire = valid && ((gst.is("move") && this.opts.immediate) || gst.is("end"))

      this.fireEventCallback()

      if (overThreshold && canFire) {
        this.fireCallback()
        this.ended = true
        clearTimeout(this.timeout)
        return
      }

      if (this.opts.timeout && !this.timeout) {
        this.timeout = timeout(this.opts.timeout, () => this.cancel())
      }
    }
  }

  /**
   * Updates the swipe recognition configuration. The new options aren't
   * merged with the old ones.
   */
  update(opts: Partial<SwipeOpts>) {
    this.opts = { ...SWIPE_DEFAULT_OPTS, ...opts }
    this.directions = Array.isArray(this.opts.direction)
      ? this.opts.direction
      : [this.opts.direction]
  }

  /** Destroys the observer. */
  destroy() {
    this.observer.destroy()
    this.reset()
  }
}
