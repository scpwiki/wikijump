import { perfy } from "./index"

/**
 * Decorator for measuring the performance of a function or method.
 * @example
 * ```
 * let perf = 0
 * class foo {
 *   ;@measure(time => perf = time)
 *   method() { return "some-expensive-calculation" }
 * }
 * ```
 * @param callback - Callback to fire with the performance measurement taken.
 */
export function measure(callback: (perf: number) => void) {
  // eslint-disable-next-line @typescript-eslint/ban-types
  return (_target: Object, _propertyKey: string, descriptor: PropertyDescriptor) => {
    const method = descriptor.value
    const async = method.constructor.name === "AsyncFunction"

    if (!async) {
      descriptor.value = function (...args: any[]) {
        const report = perfy()
        const result = method.apply(this, args)
        const perf = report()
        callback(perf)
        return result
      }
    } else {
      descriptor.value = async function (...args: any[]) {
        const report = perfy()
        const result = await method.apply(this, args)
        const perf = report()
        callback(perf)
        return result
      }
    }
  }
}
