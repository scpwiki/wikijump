import { pointsMatch, toPoints } from "@wikijump/util"
import type { Matcher, VariableTable } from "../../types"

/**
 * Matcher that takes in a list of strings, and matches each one against an
 * input to see if any match.
 */
export class LookupMatcher implements Matcher {
  /** True if matching is case-insensitive. */
  private declare ignoreCase: boolean

  /** A list of number arrays, each one being a codepoint representation of a word. */
  private declare entries: number[][]

  /** A mapping between the `entries` list and lengths of their source word. */
  private declare lengths: Map<number[], number>

  /** The longest entry in `entries`, by its original string length. */
  private declare max: number

  /**
   * @param src - The source list of strings.
   * @param ignoreCase - If `true`, matches will be case-insensitive.
   * @param variables - A variable table to use when expanding variables.
   */
  constructor(src: string[], ignoreCase?: boolean, variables?: VariableTable) {
    // longest first
    const sorted = [...src].sort((a, b) => b.length - a.length)

    this.max = sorted[0].length
    this.ignoreCase = Boolean(ignoreCase)
    this.lengths = new Map()

    this.entries = sorted.map(entry => {
      if (variables) entry = expandVariables(entry, variables)
      if (ignoreCase) entry = entry.toLowerCase()
      const points = toPoints(entry)
      this.lengths.set(points, entry.length)
      return points
    })
  }

  /**
   * Internal method which returns the length of a match against a string,
   * if one was found.
   *
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  private exec(str: string, pos: number) {
    const slice = str.slice(pos, pos + this.max)
    const against = toPoints(this.ignoreCase ? slice.toLowerCase() : slice)
    for (let i = 0; i < this.entries.length; i++) {
      const points = this.entries[i]
      if (pointsMatch(points, against, 0)) return this.lengths.get(points)!
    }
    return null
  }

  /**
   * Tests to see if a string is in this list.
   *
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  test(str: string, pos: number) {
    return this.exec(str, pos) !== null
  }

  /**
   * Returns the results of attempting to match a string against this list.
   *
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  match(str: string, pos: number) {
    const result = this.exec(str, pos)
    if (!result) return null
    return {
      total: str.slice(pos, pos + result),
      captures: null,
      length: result
    }
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
