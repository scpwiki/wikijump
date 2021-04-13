import { isAnyObject } from 'is-what'

// https://gist.github.com/hyamamoto/fd435505d29ebfa3d9716fd2be8d42f0#gistcomment-2694461
/** Very quickly generates a (non-secure) hash from the given string. */
export function hash(s: string) {
  let h = 0
  for (let i = 0; i < s.length; i++)
  	h = Math.imul(31, h) + s.charCodeAt(i) | 0
  return h
}

export interface SearchOpts {
  /** Starting minimum index for the search. */
  min?: number
  /** Starting maximum index for the search. */
  max?: number
  /** If true, the search will return the closest index to the desired value on failure. */
  precise?: boolean
}

/** Performs a binary search through an array.
 *
 *  The comparator function should return -1 if undershooting the desired value,
 *  +1 if overshooting, and 0 if the value was found.
 *
 *  The comparator can also short-circuit the search by returning true or false.
 *  Returning true is like returning a 0 (target found), but returning false induces a null return. */
export function search<T, TR>(
  haystack: T[],
  target: TR,
  comparator: (element: T, target: TR) => number | boolean,
  { min = 0, max = haystack.length - 1, precise = true }: SearchOpts = {}
) {
  let index = -1
  while (min <= max) {
    index = min + ((max - min) >>> 1)
    const cmp = comparator(haystack[index], target)
    if (cmp === true || cmp === 0) return { element: haystack[index], index }
    if (cmp === false) return null
    if (cmp < 0) { min = index + 1; continue }
    if (cmp > 0) { max = index - 1; continue }
  }

  if (!precise) return { element: null, index }

  return null
}

/** Checks if an array or object is empty. Will return true for non-objects. */
export function isEmpty(obj: any) {
  if (!obj) return true
  if (obj instanceof Array)
    return obj.length === 0
  if (obj.constructor === Object)
    return Object.keys(obj).length === 0
  return true
}

/** Creates a type that is the type of `T` if it had a known property `K`. */
type Has<K extends string, T> = T extends { [P in K]?: infer R } ? Omit<T, K> & Record<K, R> : never

/** Returns if an object `T` has a key `K`, and only returns true if the value of that key isn't undefined. */
export function has<K extends string, T>(key: K, obj: T): obj is T extends Record<any, any> ? Has<K, T> : never {
  if (!isAnyObject(obj)) return false
  return key in obj && (obj as any)[key] !== undefined
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
  return str.replace(/[.*+?^${}()|\[\]\\]/g, '\\$&')
}

/** Checks if a string has any of the provided sigils. (e.g. `hasSigil('!string', '!') -> true`) */
export function hasSigil<T extends string = string>(str: unknown, sigils: string | string[]): str is T {
  if (typeof str !== 'string') return false
  if (typeof sigils === 'string') return str.startsWith(sigils)
  for (const sigil of sigils)
    if (str.startsWith(sigil)) return true
  return false
}

/** Removes sigils from a string recursively. */
export function unSigil<T extends string>(str: T, sigils: string | string[]): T {
  if (typeof str !== 'string') throw new TypeError('str must be a string')
  if (typeof sigils === 'string')
    return (str.startsWith(sigils) ? str.slice(sigils.length) : str) as T
  else for (const sigil of sigils)
    if (str.startsWith(sigil)) return unSigil(str.slice(sigil.length), sigils) as T
  return str as T
}

/** Creates a simple pseudo-random ID, with an optional prefix attached. */
export function createID(prefix = '') {
  const suffix = hash(Math.random() * 100 + prefix)
  return prefix + '-' + suffix
}

/** Converts a string into an array of codepoints. */
export function toPoints(str: string) {
  const codes: number[] = []
  for (let i = 0; i < str.length; i++) {
    codes.push(str.codePointAt(i)!)
  }
  return codes
}

/** Checks an array of codepoints against a codepoint array or a string, starting from a given position. */
export function pointsMatch(points: number[], str: string | number[], pos: number) {
  if (typeof str === 'string') for (let i = 0; i < points.length; i++) {
    if (points[i] !== str.codePointAt(pos + i)) return false
  }
  else for (let i = 0; i < points.length; i++) {
    if (points[i] !== str[pos + i]) return false
  }
  return true
}

/** Performance measuring utility.
 *
 *  To use, execute the function and store the returned value.
 *  The returned value is a function that will end the performance timer and log the measured time to the console. */
export function perfy(meta: string, threshold: number) {
  const start = performance.now()
  return () => {
    const time = performance.now() - start
    if (time > threshold) console.log(`${meta}: ${time}ms`)
  }
}
