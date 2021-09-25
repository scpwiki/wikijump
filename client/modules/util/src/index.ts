export * from "./decorators"
export * from "./timeout"

// https://gist.github.com/hyamamoto/fd435505d29ebfa3d9716fd2be8d42f0#gistcomment-2694461
/** Very quickly generates a (non-secure) hash from the given string. */
export function hash(s: string) {
  let h = 0
  for (let i = 0; i < s.length; i++) {
    h = (Math.imul(31, h) + s.charCodeAt(i)) | 0
  }
  return h
}

export interface SearchOpts {
  /** Starting minimum index for the search. */
  min?: number
  /** Starting maximum index for the search. */
  max?: number
  /**
   * If true, the search will return the closest index to the desired value
   * on failure.
   */
  precise?: boolean
}

/**
 * Performs a binary search through an array.
 *
 * The comparator function should return -1 if undershooting the desired
 * value, +1 if overshooting, and 0 if the value was found.
 *
 * The comparator can also short-circuit the search by returning true or
 * false. Returning true is like returning a 0 (target found), but
 * returning false induces a null return.
 */
export function search<T, TR>(
  haystack: T[],
  target: TR,
  comparator: (element: T, target: TR) => number | boolean,
  { min = 0, max = haystack.length - 1, precise = true }: SearchOpts = {}
) {
  if (haystack.length === 0) return null

  let index = -1
  while (min <= max) {
    index = min + ((max - min) >>> 1)
    const cmp = comparator(haystack[index], target)
    if (cmp === true || cmp === 0) return { element: haystack[index], index }
    if (cmp === false) return null
    if (cmp < 0) min = index + 1
    else if (cmp > 0) max = index - 1
  }

  if (index === -1) return null

  if (!precise) return { element: null, index }

  return null
}

/** Checks if an array or object is empty. Will return true for non-objects. */
export function isEmpty(obj: any) {
  if (!obj) return true
  if (obj instanceof Array) return obj.length === 0
  if (obj.constructor === Object) return Object.keys(obj).length === 0
  return true
}

/** Creates a type that is the type of `T` if it had a known property `K`. */
type Has<K extends string, T> = T extends { [P in K]?: infer R }
  ? Omit<T, K> & Record<K, R>
  : never

/**
 * Returns if an object `T` has a key `K`, and only returns true if the
 * value of that key isn't undefined.
 */
export function has<K extends string, T>(
  key: K,
  obj: T
): obj is T extends Record<any, any> ? Has<K, T> : never {
  if (typeof obj !== "object") return false
  // @ts-ignore
  return key in obj && obj[key] !== undefined
}

/** Removes all properties assigned to `undefined` in an object. */
export function removeUndefined<T>(obj: T) {
  // this wacky approach is faster as it avoids an iterator
  const keys = Object.keys(obj) as (keyof T)[]
  for (let i = 0; i < keys.length; i++) {
    if (obj[keys[i]] === undefined) delete obj[keys[i]]
  }
  return obj as { [K in keyof T]: Exclude<T[K], undefined> }
}

/** Takes a string and escapes any `RegExp` sensitive characters. */
export function escapeRegExp(str: string) {
  return str.replace(/[.*+?^${}()|\[\]\\]/g, "\\$&")
}

/**
 * Checks if a string has any of the provided sigils.
 *
 * @example
 *
 * ```ts
 * hasSigil("!string", "!") // true
 * ```
 */
export function hasSigil<T extends string = string>(
  str: unknown,
  sigils: string | string[]
): str is T {
  if (typeof str !== "string") return false
  if (typeof sigils === "string") return str.startsWith(sigils)
  for (const sigil of sigils) if (str.startsWith(sigil)) return true
  return false
}

/** Removes sigils from a string recursively. */
export function unSigil<T extends string>(str: T, sigils: string | string[]): T {
  if (typeof sigils === "string") {
    return (str.startsWith(sigils) ? str.slice(sigils.length) : str) as T
  } else {
    for (const sigil of sigils) {
      if (str.startsWith(sigil)) {
        return unSigil(str.slice(sigil.length), sigils) as T
      }
    }
  }
  return str as T
}

/** Creates a simple pseudo-random ID, with an optional prefix attached. */
export function createID(prefix = "") {
  const suffix = hash(Math.random() * 100 + prefix)
  return `${prefix}-${suffix}`
}

/** Converts a string into an array of codepoints. */
export function toPoints(str: string) {
  const codes: number[] = []
  for (let i = 0; i < str.length; i++) {
    codes.push(str.codePointAt(i)!)
  }
  return codes
}

/**
 * Checks an array of codepoints against a codepoint array or a string,
 * starting from a given position.
 */
export function pointsMatch(points: number[], str: string | number[], pos: number) {
  if (typeof str === "string") {
    for (let i = 0; i < points.length; i++) {
      if (points[i] !== str.codePointAt(pos + i)) return false
    }
  } else {
    for (let i = 0; i < points.length; i++) {
      if (points[i] !== str[pos + i]) return false
    }
  }
  return true
}

/**
 * Performance measuring utility.
 *
 * To use, execute the function and store the returned value. The returned
 * value is a function that will end the performance timer and log the
 * measured time to the console.
 */
export function perfy(meta?: string, threshold?: number): (msg?: string) => number {
  const start = performance.now()
  return (msg?: string) => {
    const time = parseFloat((performance.now() - start).toFixed(4))
    if (meta && threshold && time > threshold) {
      if (msg) {
        console.log(`${msg} | ${meta}: ${time}ms`)
      } else {
        console.log(`${meta}: ${time}ms`)
      }
    }
    return time
  }
}

/** Returns a promise that resolves after the specified number of miliseconds. */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Creates and returns a promise that resolves when an invokation of
 * `requestAnimationFrame()` fires its callback.
 */
export function animationFrame(): Promise<number> {
  return new Promise(resolve => requestAnimationFrame(resolve))
}

// Credit: https://gist.github.com/beaucharman/e46b8e4d03ef30480d7f4db5a78498ca
// Personally, I think this is one of the more elegant JS throttle functions.
/**
 * Returns a 'throttled' variant of the given function. This function will
 * only be able to execute every `limitMS` ms. Use to rate-limit functions
 * for performance. You can have the first call be immediate by setting the
 * third parameter to `true`.
 */
export function throttle<T extends AnyFunction>(
  fn: T,
  limitMS: number,
  immediate = false
) {
  let timeout: number | null = null
  let initialCall = true

  return function (this: any, ...args: Parameters<T>) {
    const callNow = immediate && initialCall
    const next = () => {
      // @ts-ignore
      fn.apply(this, [...args])
      timeout = null
    }
    if (callNow) {
      initialCall = false
      next()
    }
    if (!timeout) timeout = setTimeout(next, limitMS) as unknown as number
  }
}

// Credit: https://gist.github.com/vincentorback/9649034
/** Returns a 'debounced' variant of the given function. */
export function debounce<T extends AnyFunction>(fn: T, wait = 1) {
  let timeout: any
  return function (this: any, ...args: Parameters<T>) {
    clearTimeout(timeout)
    timeout = setTimeout(() => void fn.call(this, ...args), wait)
  }
}

/**
 * Waits until the specified function returns `true`. It will call the
 * specified async function to determine the polling interval. If none is
 * given, it will poll every 100ms.
 */
export async function waitFor(
  conditionFn: () => Promisable<boolean>,
  asyncTimerFn: () => Promise<void> = () => sleep(100)
) {
  while ((await conditionFn()) === false) {
    await asyncTimerFn()
    continue
  }
  return true
}

/**
 * Returns a new 'locked' async function, constructed using the specified
 * function. A locked asynchronous function will only allow a singular
 * instance of itself to be running at one time.
 */
export function createLock<T extends AnyFunction>(fn: T) {
  type Return = PromiseValue<ReturnType<T>>
  const call = async (args: any[]) => {
    return (await fn(...args)) as Return
  }

  let running: Promise<Return> | null = null

  return async (...args: Parameters<T>) => {
    if (running) await running
    running = call(args)
    const result = await running
    running = null
    return result
  }
}

/**
 * Returns a new 'locked' async function, constructed using the specified
 * function. A locked asynchronous function will only allow a singular
 * instance of itself to be running at one time.
 *
 * Additional calls will return null, but they will signal to the original,
 * still running call to "restart" with the new given value. This means
 * that the original call will only ever return the most freshly sourced result.
 */
export function createMutatingLock<T extends AnyFunction>(fn: T) {
  type Return = PromiseValue<ReturnType<T>>
  const call = async (args: any[]) => {
    return (await fn(...args)) as Return
  }

  let running: boolean
  let useArgs: any[] = []
  return async (...args: Parameters<T>): Promise<Return | null> => {
    useArgs = args
    if (running) return null
    running = true
    let result = await call(args)
    // loop to catch if other calls mutate the arguments
    // if they don't this gets skipped
    while (useArgs !== args) {
      // @ts-ignore
      args = useArgs
      result = await call(args)
    }
    useArgs = []
    running = false
    return result
  }
}

/**
 * Returns a function that will be "queued" to execute only on animation
 * frames. Calling multiple times will run only once on the next
 * requestAnimationFrame.
 *
 * @example
 *
 * ```ts
 * const func = createAnimQueued(function target(args) => { 'foo' })
 * func()
 * func() // doesn't run as the previous call is already queued
 * ```
 */
export function createAnimQueued<T extends AnyFunction>(fn: T) {
  let queued: boolean
  let useArgs: any[] = []
  return (...args: Parameters<T>): void => {
    useArgs = args
    if (queued !== true) {
      queued = true
      requestAnimationFrame(async () => {
        // @ts-ignore
        await fn(...useArgs)
        queued = false
      })
    }
  }
}

const HAS_IDLE_CALLBACK = "requestIdleCallback" in globalThis

/** Safely calls `requestIdleCallback` in an awaitable `Promise`. */
// bad coverage as requestIdleCallback isn't always available
/*! c8 ignore next */
export function idleCallback<T extends AnyFunction<any>>(
  cb: T,
  timeout = 100
): Promise<ReturnType<T>> {
  if (!HAS_IDLE_CALLBACK) {
    return new Promise(resolve => setTimeout(() => resolve(cb()), timeout))
  } else {
    return new Promise(resolve =>
      // @ts-ignore
      requestIdleCallback(() => resolve(cb()), { timeout })
    )
  }
}

/**
 * See `createAnimQueued` for a description of how this function works. The
 * only difference is that this function uses `requestIdleCallback`
 * instead. If `requestIdleCallback` isn't available, it will use
 * `createAnimQueued` instead.
 *
 * @see {@link createAnimQueued}
 */
// bad coverage as requestIdleCallback isn't always available
/*! c8 ignore next */
export function createIdleQueued<T extends AnyFunction>(fn: T, timeout = 100) {
  if (!HAS_IDLE_CALLBACK) return createAnimQueued(fn)
  let queued: boolean
  let useArgs: any[] = []
  return (...args: Parameters<T>): void => {
    useArgs = args
    if (queued !== true) {
      queued = true
      // @ts-ignore
      requestIdleCallback(
        async () => {
          // @ts-ignore
          await fn(...useArgs)
          queued = false
        },
        { timeout }
      )
    }
  }
}

// so we can load this module in workers:
let domParser: DOMParser
try {
  domParser = new DOMParser()
} catch {}

/** Takes a string of HTML and creates a {@link DocumentFragment}. */
export function toFragment(html: string) {
  const parsed = domParser.parseFromString(html, "text/html")
  const fragment = document.createDocumentFragment()
  fragment.append(...Array.from(parsed.body.children))
  return fragment
}

/**
 * **DOES NOT ESCAPE INPUT**
 *
 * Template string tag that creates a {@link DocumentFragment}.
 */
export function html(strings: TemplateStringsArray, ...subs: (string | string[])[]) {
  const src = strings.raw.reduce((prev, cur, idx) => {
    let sub = subs[idx - 1]
    if (Array.isArray(sub)) sub = sub.join("")
    return prev + sub + cur
  })
  return toFragment(src)
}

/**
 * Performs a modulo operation. This differs from JavaScript's `%`
 * operator, which is more of a remainder operator.
 *
 * @param a - The dividend.
 * @param n - The divisor.
 */
export function mod(a: number, n: number) {
  return ((a % n) + n) % n
}

/**
 * Replaces a range inside of a string with a substitute.
 *
 * @param str - The string which should have a range inside of it replaced.
 * @param from - The start of the replacement range.
 * @param to - The end of the replacement range.
 * @param sub - The replacement/substitute string.
 */
export function replaceRange(str: string, from: number, to: number, sub: string) {
  return str.substr(0, from) + sub + str.substr(to)
}

/**
 * Uppercases a string.
 *
 * @param str - The string to uppercase.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function uppercase(str: string, locale?: string | string[]) {
  return locale ? str.toLocaleUpperCase(locale) : str.toUpperCase()
}

/**
 * Lowercases a string.
 *
 * @param str - The string to lowercase.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function lowercase(str: string, locale?: string | string[]) {
  return locale ? str.toLocaleLowerCase(locale) : str.toLowerCase()
}

/**
 * Titlecases a string.
 *
 * @param str - The string to titlecase.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function titlecase(str: string, locale?: string | string[]) {
  return replaceRange(lowercase(str, locale), 0, 1, uppercase(str[0], locale))
}

/**
 * Determines if a string is titlecased.
 *
 * @param str - The string to check.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function isTitlecased(str: string, locale?: string | string[]) {
  return uppercase(str[0], locale) === str[0]
}

/**
 * Determines if a string is completely uppercased.
 *
 * @param str - The string to check.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function isUppercased(str: string, locale?: string | string[]) {
  return uppercase(str, locale) === str
}

/**
 * Determines if a string is completely lowercased.
 *
 * @param str - The string to check.
 * @param locale - Uses a locale, or a list of locales, case mapping if
 *   provided. This usually won't be needed, as JS tries to account for
 *   non-ASCII/Latin text when handling casing.
 */
export function isLowercased(str: string, locale?: string | string[]) {
  return lowercase(str, locale) === str
}

/** Helper for turning a relative `?url` import into an absolute path. */
export async function url(imp: Promise<any>) {
  return new URL((await imp).default, import.meta.url).toString()
}

/**
 * Deduplicates an array. Does not mutate the original array.
 *
 * @param arr - The array to deduplicate.
 * @param insert - Additional values to insert into the array, if desired.
 */
export function dedupe<T extends any[]>(arr: T, ...insert: T) {
  return [...new Set([...arr, ...insert])] as T
}
