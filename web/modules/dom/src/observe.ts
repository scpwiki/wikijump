const OBSERVER_CONFIG = {
  childList: true,
  subtree: true,
  attributes: true,
  characterData: true
}

/**
 * Starts observing a target element using a `MutationObserver`.
 *
 * @param target - The target element to observe.
 * @param callback - The callback to call when the target element changes.
 */
export function observe(
  target: HTMLElement,
  callback: (changes: MutationRecord[]) => void
) {
  const observer = new MutationObserver(callback)
  observer.observe(target, OBSERVER_CONFIG)
  return observer
}

/**
 * Decorator that pauses an element's `MutationObserver` during a method
 * call. The observer needs to be in a public property named `observer`.
 */
export function pauseObservation(
  target: HTMLElement & { observer: MutationObserver },
  _key: string,
  descriptor: PropertyDescriptor
) {
  const method = descriptor.value
  const async = method.constructor.name === "AsyncFunction"

  if (async) {
    // counts how many instances of the function are running to prevent races
    let runCount = 0

    descriptor.value = async function (this: typeof target, ...args: any[]) {
      this.observer.disconnect()
      runCount++
      const result = await method.apply(this, args)
      runCount--
      if (!runCount) this.observer.observe(this, OBSERVER_CONFIG)
      return result
    }
  } else {
    descriptor.value = function (this: typeof target, ...args: any[]) {
      this.observer.disconnect()
      const result = method.apply(this, args)
      this.observer.observe(this, OBSERVER_CONFIG)
      return result
    }
  }
}
