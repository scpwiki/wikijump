import { re } from "../util"

/**
 * A table of conversions (either input to output, or output to input) for
 * a word. Uses the following syntax:
 *
 * ```text
 * # for input-output conversion
 * ICONV <number of entries>
 * ICONV <pattern> <replacement>
 *
 * # for output-input conversion
 * OCONV <number of entries>
 * OCONV <pattern> <replacement>
 * ```
 *
 * Typically, a `pattern` and its `replacement` are just simple strings.
 * This feature is usually used just for normalizing strings containing
 * unusual typographics, like trigraphs and fancy apostrophes.
 *
 * `pattern` does have an unusual undocumented syntax feature: If the `_`
 * underscore character is in the pattern, it may be transformed in one of
 * three ways:
 *
 * - At the start of the `pattern`: Denotes a `RegExp` `^` anchor.
 * - At the end of the `pattern`: Denotes a `RegExp` `$` anchor.
 * - Everywhere else: Entirely ignored.
 *
 * Due to this undocumented feature, patterns are compiled to `RegExp`
 * rather than left as simple strings.
 */
export class ConvTable {
  /** The list of patterns and their replacements. */
  declare table: { pattern: RegExp; replacement: string }[]

  /**
   * @param pairs - A list of string pair tuples that denotes a pattern and
   *   its replacement.
   */
  constructor(pairs: [string, string][]) {
    this.table = pairs.map(([pattern, replacement]) => {
      const cleanedPattern = pattern
        .replace(/^_/, "^")
        .replace(/_$/, "$")
        .replaceAll("_", "")
      return {
        pattern: re`/${cleanedPattern}/y`,
        replacement: replacement.replaceAll("_", " ")
      }
    })
  }

  /**
   * Applies this table's rules against a word and returns the resulting
   * string. Uses the following algorithm, to prevent recursion:
   *
   * - For each position in the word:
   * - ...find any matching rules
   * - ...choose the one that has the longest pattern
   * - ...apply its replacement, and shift the current position past the replacement
   *
   * @param word - The word to transform.
   */
  match(word: string) {
    let pos = 0
    let res = ""
    while (pos < word.length) {
      let replacement: string | null = null
      let len = 0
      let max = 0

      for (const pair of this.table) {
        pair.pattern.lastIndex = pos
        if (pair.pattern.source.length > max && pair.pattern.test(word)) {
          pair.pattern.lastIndex = pos
          len = pair.pattern.exec(word)![0].length
          max = pair.pattern.source.length
          replacement = pair.replacement
        }
      }

      if (replacement) {
        res += replacement
        pos += len
      } else {
        res += word[pos]
        pos += 1
      }
    }

    return res
  }
}
