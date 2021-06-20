import iterate from "iterare"
import { CONSTANTS as C } from "./constants"

/**
 * Creates a `RegExp` using a string template. Supports flags.
 *
 * @example
 *
 * ```ts
 * const regex = re`/abc/g`
 * ```
 */
export function re(strings: TemplateStringsArray, ...keys: any[]) {
  const split = C.SPLIT_REGEX_REGEX.exec(String.raw(strings, ...keys))
  if (!split) throw new SyntaxError()
  const [, , src = "", flags = ""] = split
  return new RegExp(src, flags)
}

/**
 * Helper for checking if a `Set`, `Array`, or `string` contains another
 * `string`. Handles undefined or null inputs by returning false.
 *
 * @param value - The value to check for.
 * @param container - The container of strings (or another string).
 */
export function includes(
  value: string | undefined | null,
  container: Set<string> | string[] | string | undefined | null
) {
  if (value === undefined || value === null) return false
  if (container === undefined || container === null) return false

  if (typeof container === "string" || Array.isArray(container)) {
    return container.includes(value)
  } else {
    return container.has(value)
  }
}

/** Takes a string and escapes any `RegExp` sensitive characters. */
export function escapeRegExp(str: string) {
  return str.replace(/[.*+?^${}()|\[\]\\]/g, "\\$&")
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
  return titlecase(str, locale) === str
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

/**
 * Reverses a string.
 *
 * @param str - The string to reverse.
 */
export function reverse(str: string) {
  return str.split("").reverse().join("")
}

/** Splits a line by its whitespace. */
export function split(line: string) {
  return line.split(C.SPLIT_LINE_REGEX)
}

/**
 * Returns a new set containing all intersecting elements between two sets.
 *
 * @param a - The first set.
 * @param b - The second set.
 */
export function intersect<T>(a: Set<T>, b: Set<T>) {
  return iterate(a)
    .filter(x => b.has(x))
    .toSet()
}

export function concat(a: string, b: Iterable<string>): string
export function concat<T>(a: Set<T>, b: Iterable<T>): Set<T>
export function concat<T>(a: T[], b: Iterable<T>): T[]
export function concat<T>(a: Iterable<T>, b: Iterable<T>): string | Set<T> | T[] {
  const iter = iterate(a).concat(b)
  if (typeof a === "string") return iter.join("")
  if (a instanceof Set) return iter.toSet()
  if (Array.isArray(a)) return iter.toArray()
  throw new TypeError("Unknown iterable given!")
}

// https://gist.github.com/cybercase/db7dde901d7070c98c48#gistcomment-3718142
type Iterableify<T> = { [K in keyof T]: Iterable<T[K]> }
export function* product<T extends unknown[]>(
  ...iterables: Iterableify<T>
): Generator<T> {
  if (iterables.length === 0) {
    return
  }
  const iterators = iterables.map(it => it[Symbol.iterator]())
  const results = iterators.map(it => it.next())

  // Cycle through iterators
  for (let i = 0; ; ) {
    if (results[i].done) {
      // Reset the current iterator
      iterators[i] = iterables[i][Symbol.iterator]()
      results[i] = iterators[i].next()
      // Advance and exit if we've reached the end
      if (++i >= iterators.length) {
        return
      }
    } else {
      yield results.map(({ value }) => value) as T
      i = 0
    }
    results[i] = iterators[i].next()
  }
}

/** Returns true if the given iterator yields literally anything. */
export function any(gen: Iterable<unknown>) {
  for (const _ of gen) {
    return true
  }
  return false
}

/**
 * Returns the number of characters common between two strings, in both
 * type and position.
 */
export function commonCharacters(s1: string, s2: string) {
  return [...s1].filter((ch, index) => ch === s2[index]).length
}

/** Returns the amount of characters in common between the left-sides of two strings. */
export function leftCommonSubstring(s1: string, s2: string) {
  for (let i = 0; i < Math.max(s1.length, s2.length); i++) {
    if (s1[i] !== s2[i]) return i
  }
  return 0
}

/**
 * Returns the number of ngrams of `s1` are in `s2`. Higher is better.
 *
 * @param max - The `n` in `ngram`.
 * @param s1 - String to compare against `s2`.
 * @param s2 - String to compare against `s1`.
 * @param weighted - Reduce score depending on number ngrams *not contained* in `s2`.
 * @param longerIsWorse - Reduce score when `s2` is longer than `s1`.
 * @param anyMismatch - Reduce score if the strings differ in length at all.
 */
export function ngram(
  max: number,
  s1: string,
  s2: string,
  weighted = false,
  anyMismatch = false,
  longerIsWorse = false
) {
  const l1 = s1.length
  const l2 = s2.length
  if (l2 === 0) return 0

  let nscore = 0

  for (let size = 0; size < max + 1; size++) {
    let ns = 0
    for (let pos = 0; pos < l1 - (size + 1); pos++) {
      if (s2.includes(s1.slice(pos, pos + size))) {
        ns++
      } else if (weighted) {
        ns--
        if (pos === 0 || pos + size === l1) {
          ns--
        }
      }
    }

    nscore += ns

    if (ns < 2 && !weighted) break
  }

  let penalty = 0
  if (longerIsWorse) {
    penalty = l2 - l1 - 2
  } else if (anyMismatch) {
    penalty = Math.abs(l2 - l1) - 2
  }

  return penalty > 0 ? nscore - penalty : nscore
}

/** Length of the "longest common subsequence" in two strings. */
export function lcslen(a: string, b: string) {
  let m = a.length
  let n = b.length
  let C: number[][] = []
  let i: number
  let j: number

  for (i = 0; i <= m; i++) C.push([0])
  for (j = 0; j < n; j++) C[0].push(0)
  for (i = 0; i < m; i++) {
    for (j = 0; j < n; j++) {
      C[i + 1][j + 1] = a[i] === b[j] ? C[i][j] + 1 : Math.max(C[i + 1][j], C[i][j + 1])
    }
  }

  return C[m - 1][n - 1]
}
