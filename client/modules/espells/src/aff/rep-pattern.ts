import { re, replaceRange } from "../util"

/**
 * A replacement pattern describes a pattern and its replacement, with the
 * pattern matching a common typo in a word, and the replacement being the
 * relevant correction for that typo. It uses the following syntax:
 *
 * ```text
 * REP <number of entries>
 * REP <pattern> <replacement>
 * ```
 *
 * The `pattern` syntax supports `^` and `$` anchors, like `RegExp`. In the
 * `replacement` string, a `_` underscore can be used in the string to
 * represent a space, e.g. correcting `alot` to `a lot`, using the
 * `replacement` `a_lot`.
 */
export class RepPattern {
  /** The `RegExp` pattern to replace. */
  declare pattern: RegExp
  /** The string to replace anything matched by `pattern` with. */
  declare replacement: string

  /**
   * @param pattern - The pattern to replace. Is treated as a `RegExp`, but
   *   given as a string.
   * @param replacement - The string to replace anything matched by `pattern` with.
   */
  constructor(pattern: string, replacement: string) {
    this.pattern = re`/${pattern}/g`
    this.replacement = replacement.replaceAll("_", " ")
  }

  /** Yields the permutations of a word with this `RepPattern` applied to it. */
  *replace(word: string): Iterable<string> {
    for (const match of word.matchAll(this.pattern)) {
      const from = match.index!
      const to = from + this.replacement.length
      const replaced = replaceRange(word, from, to, this.replacement)
      yield replaced
    }
  }
}
