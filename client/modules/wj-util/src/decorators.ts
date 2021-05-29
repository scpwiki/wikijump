import { perfy } from "./index"

/**
 * Decorator for measuring the performance of a function or method.
 *
 * @example
 *
 * ```ts
 * let perf = 0
 * class foo {
 *   @measure(time => (perf = time))
 *   method() {
 *     return "some-expensive-calculation"
 *   }
 * }
 * ```
 *
 * @param callback - Callback to fire with the performance measurement taken.
 */
export function measure(callback: (perf: number, name: string) => void) {
  // eslint-disable-next-line @typescript-eslint/ban-types
  return (_target: Object, propertyKey: string, descriptor: PropertyDescriptor) => {
    const method = descriptor.value
    const async = method.constructor.name === "AsyncFunction"

    if (async) {
      descriptor.value = async function (...args: any[]) {
        const report = perfy()
        const result = await method.apply(this, args)
        const perf = report()
        callback(perf, propertyKey)
        return result
      }
    } else {
      descriptor.value = function (...args: any[]) {
        const report = perfy()
        const result = method.apply(this, args)
        const perf = report()
        callback(perf, propertyKey)
        return result
      }
    }
  }
}

/** Decorator for logging the performance of a function. */
export const logPerformance = measure((time, name) => {
  if (time >= 1) console.log(`${name}: ${time}ms`)
})
