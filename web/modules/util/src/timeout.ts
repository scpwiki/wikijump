/**
 * Replacement class for `setTimeout`, with tons of utility features.
 * Additionally provides some extra safety and avoids typing issues with
 * `NodeJS.Timeout`.
 */
export class Timeout<T = void> {
  // typed as any to avoid NodeJS.Timeout
  private declare timeout: any

  /** Function that resolves the timeout's `promise` Promise. */
  private declare promiseResolve: (resolved: T) => void

  /** The delay given before the callback should be fired. */
  declare delay: number

  /** The time the timeout will end by. */
  declare ends: Date

  /** The time the timeout was started. */
  declare started: Date

  /**
   * A promise that resolves when the timeout expires. If the timeout is
   * reset after it has expired, this property will be updated, so make
   * sure to access this property directly and do not store it.
   */
  declare promise: Promise<T>

  /**
   * The final value returned by the callback function. Always undefined if
   * the timeout is running.
   */
  declare value?: T

  /**
   * The callback that will be fired when the timeout expires. This **will
   * not** be the same function that was given when this timeout was
   * constructed, so identity comparisons won't work.
   */
  private declare cb?: () => void

  /**
   * @param delay - The delay for the timer. Set to `null` for an immediate timer.
   * @param cb - The callback that will be fired when the timeout expires.
   *   Is optional.
   * @param immediate - If true, the callback will be fired immediately.
   *   Defaults to true.
   */
  constructor(delay?: number | null, cb?: (() => T) | null, immediate = true) {
    this.reset(delay, cb)
    if (!immediate) this.clear()
  }

  /** True if the timeout is running. */
  get running() {
    return this.timeout !== undefined
  }

  /** Function for fulfilling the thenable contract. */
  then(resolve?: (value: T) => T | PromiseLike<T>) {
    return this.promise.then(resolve)
  }

  /** The amount of time remaining before the timeout expires, in milliseconds. */
  remaining() {
    if (!this.ends || !this.started) return 0
    const remaining = this.ends.getTime() - new Date().getTime()
    return remaining > 0 ? remaining : 0
  }

  // apparently, this is how you do typeguards for classes?
  // it's a bit weird, it appears like it's a shorthand for
  // `this is Timeout<T> & { value: T }
  /** Returns true if the timeout has expired already. */
  expired(): this is { value: T } {
    return this.timeout === undefined
  }

  /** Clears the timeout and prevents it from expiring. */
  clear() {
    if (!this.timeout) return
    clearTimeout(this.timeout)
    this.timeout = undefined
  }

  /**
   * Extends the timeout, adding the given delay to the current time
   * remaining. Does nothing if the timeout has already expired.
   *
   * @param delay - The delay to add to the current time remaining.
   */
  extend(delay: number) {
    if (this.expired()) return
    this.reset(this.remaining() + delay)
  }

  /**
   * Resets the timeout. Optionally allows changing the delay and callback.
   *
   * @param delay - The delay between now and when the callback should be
   *   fired. Set to `null` to have a "tick" timer.
   * @param cb - The callback that will be fired when the timeout expires.
   *   Provide `null` to get rid of the current callback.
   */
  reset(delay?: number | null, cb?: (() => T) | null) {
    if (cb && typeof cb !== "function") {
      console.error("Avoided potential string eval in timeout!")
      throw new Error("Timeout callback must be a function")
    }

    if (this.expired() || !this.promise) {
      this.promise = new Promise<T>(resolve => {
        this.promiseResolve = resolve
      })
    }

    if (typeof delay === "number") this.delay = delay
    else if (delay === null) this.delay = 0

    if (cb) {
      this.cb = () => {
        const out = cb()
        this.value = out
        this.promiseResolve(out)
      }
    } else if (cb === null) {
      this.cb = undefined
    }

    if (this.expired()) this.started = new Date()
    this.ends = new Date(this.started.getTime() + this.delay)
    this.value = undefined
    this.clear() // make sure we end the old timeout

    this.timeout = setTimeout(() => {
      if (this.cb) this.cb()
      this.timeout = undefined
    }, this.delay)
  }
}

/**
 * Creates a new {@link Timeout}.
 *
 * @param delay - The delay between now and when the callback should be fired.
 * @param cb - The callback that will be fired when the timeout expires.
 */
export function timeout<T>(delay: number, cb?: () => T) {
  return new Timeout(delay, cb)
}

/**
 * Creates a new {@link Timeout} that resolves as soon as possible.
 *
 * @param cb - The callback that will be fired when the timeout expires.
 */
export function tick<T>(cb?: () => T) {
  return new Timeout(0, cb)
}

/**
 * Clears a {@link Timeout}.
 *
 * @param timeout - The timeout to clear.
 */
function clearTimeoutClass(timeout: Timeout | undefined | null) {
  if (!timeout) return
  timeout.clear()
}

export { clearTimeoutClass as clearTimeout }

/** Symbol for identifying timed out promises. */
export const TIMED_OUT_SYMBOL = Symbol("timedout")

/**
 * Utility for handling promise timeouts. If the {@link TIMED_OUT_SYMBOL}
 * symbol is returned, the promise timed out.
 *
 * @param promise - The promise to wrap.
 * @param time - The duration to use.
 */
export async function timedout<T>(promise: Promise<T>, time: number) {
  const timer = timeout<typeof TIMED_OUT_SYMBOL>(time, () => TIMED_OUT_SYMBOL)
  const result = await Promise.race([promise, timer.promise])
  if (result !== TIMED_OUT_SYMBOL) timer.clear()
  return result
}
