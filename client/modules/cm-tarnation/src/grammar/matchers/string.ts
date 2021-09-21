import { pointsMatch, toPoints } from "@wikijump/util"
import type { Matcher, VariableTable } from "../types"

/** Matcher that takes a string pattern, and matches that against an input. */
export class StringMatcher implements Matcher {
  /** Internal codepoint representation of the pattern. */
  private declare points: number[]

  /** The original length of the pattern. */
  private declare length: number

  /** If true, the pattern has to check for casing when matching. */
  private declare cased: boolean

  /**
   * @param src - The source string.
   * @param ignoreCase - If `true`, matches will be case-insensitive.
   * @param variables - A variable table to use when expanding variables.
   */
  constructor(str: string, ignoreCase = false, variables?: VariableTable) {
    if (variables) str = expandVariables(str, variables)
    if (ignoreCase) str = str.toLowerCase()
    this.cased = ignoreCase && str.toLowerCase() !== str.toUpperCase()
    this.length = str.length
    this.points = toPoints(str)
  }

  /**
   * Tests to see if this pattern matches the given string.
   *
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  test(str: string, pos: number) {
    if (this.length > str.length) return false

    if (pointsMatch(this.points, str, pos)) return true

    // less optimized code path due to lowercasing requirement
    if (this.cased) {
      const lowered = str.slice(pos, pos + this.length).toLowerCase()
      return pointsMatch(this.points, lowered, 0)
    }

    return false
  }

  /**
   * Returns the results of attempting to match a string against this pattern.
   *
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  match(str: string, pos: number) {
    if (this.test(str, pos)) {
      return {
        total: str.slice(pos, pos + this.length),
        captures: null,
        length: this.length
      }
    }
    return null
  }
}

/**
 * Expands the variables in a pattern source using a variable table.
 *
 * @param src - The source to expand.
 * @param variables - The variable table to use.
 */
function expandVariables(src: string, variables: VariableTable) {
  let depth = 0
  while (/@\w/.test(src) && depth < 5) {
    depth++
    src = src.replace(/@(\w+)/g, (_, ident: string) => {
      const value = variables[ident]
      if (typeof value !== "string") {
        throw new Error(`Variable ${ident} is not a string`)
      }
      return value
    })
  }

  return src
}
