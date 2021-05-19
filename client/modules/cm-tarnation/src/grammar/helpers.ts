import { pointsMatch, toPoints } from "wj-util"
import type * as DF from "./definition"

const REGEX_SPLIT = /^([^]*)\/([^]+)\/([^]*)$/

/**
 * Safely compiles a regular expression.
 *
 * @example
 *
 * ```ts
 * // returns null if features aren't supported (e.g. Safari)
 * const regex = re`/(?<=\d)\w+/d`
 * ```
 */
export function re(str: TemplateStringsArray) {
  const split = REGEX_SPLIT.exec(str.raw[0])
  if (!split) return null
  const [, , src = "", flags = ""] = split
  if (!src) return null
  try {
    return new RegExp(src, flags)
  } catch {
    return null
  }
}

/**
 * Creates a highly efficient "lookup" matching function for a list of strings.
 *
 * The function does not match the entire string. Instead, it will find the
 * first match for the start of the string.
 *
 * It's safe to use overlap values - longer strings are tried before shorter ones.
 */
export function lkup(arr: string[]): DF.MatchFunction {
  if (arr.length === 0) throw new Error("Empty string array!")

  // longest string first
  const sorted = [...arr].sort((a, b) => b.length - a.length)
  const max = sorted[0].length

  const set = new Set<number[]>(sorted.map(str => toPoints(str)))

  return (_cx, str, pos) => {
    const against = toPoints(str.slice(pos, pos + max))
    for (const points of set) {
      if (pointsMatch(points, against, 0)) {
        return [String.fromCodePoint(...points)]
      }
    }
    return null
  }
}

type CompiledMatcher = ((input: string, pos: number) => boolean) | null

/** Creates a {@link CompiledMatcher} from a `RegExp` or a `string`. */
function createMatcher(str: string, behind = false): CompiledMatcher {
  try {
    const split = REGEX_SPLIT.exec(str)

    // compile string
    if (!split) {
      if (str) {
        if (str.startsWith("\\!")) str = str.slice(2)
        const negated = str.startsWith("!")
        if (negated) str = str.slice(1)

        const len = behind ? str.length : 0
        const points = toPoints(str)

        return (input, pos) => {
          const test = pointsMatch(points, input, pos - len)
          return negated ? !test : test
        }
      }
      return null
    }

    // compile regex
    const [, offsetStr = "", src = "", flags = ""] = split
    if (!src) return null

    const negated = offsetStr.startsWith("!")
    const offset = behind ? parseInt(negated ? offsetStr.slice(1) : offsetStr) : 0
    const regex = new RegExp(src, flags + (flags.indexOf("y") !== -1 ? "" : "y"))

    return (input, pos) => {
      regex.lastIndex = pos - offset
      const test = regex.test(behind ? input.slice(0, pos) : input)
      return negated ? !test : test
    }
  } catch {
    return null
  }
}

/**
 * Lookahead utility. Given a `RegExp` or `string` (in `string` form, keep
 * in mind), it will create a function that will determine if the input
 * ahead of the search position matches.
 *
 * Lead the `RegExp` or `string` with a `!` to negate the result. `RegExp`
 * inputs can be given flags.
 *
 * @example
 *
 * ```ts
 * matcher = la`foo`
 * matcher = la`/foo\w+/i`
 * matcher = la`!foo`
 * matcher = la`!/foo\w+/i`
 * ```
 */
export function la({ raw: [str] }: TemplateStringsArray): DF.MatchFunction | null {
  const matcher = createMatcher(str)
  if (!matcher) return null
  return (_cx, input, pos) => (matcher(input, pos) ? [] : null)
}

/**
 * Lookbehind utility. Given a `RegExp` or `string` (in `string` form, keep
 * in mind), it will create a function that will determine if the input
 * behind of the search position matches.
 *
 * Unlike a `RegExp` lookbehind, the compiled function cannot traverse
 * infinitely far behind. Instead, it must be given an offset. This is
 * given in front of the `/` character.
 *
 * Lead the `RegExp` or `string` with a `!` to negate the result. `RegExp`
 * inputs can be given flags.
 *
 * @example
 *
 * ```ts
 * matcher = lb`foo`
 * matcher = lb`3/foo/i`
 * matcher = lb`!foo`
 * matcher = lb`!3/foo/i`
 * ```
 */
export function lb({ raw: [str] }: TemplateStringsArray): DF.MatchFunction | null {
  const matcher = createMatcher(str, true)
  if (!matcher) return null
  return (_cx, input, pos) => (matcher(input, pos) ? [] : null)
}
