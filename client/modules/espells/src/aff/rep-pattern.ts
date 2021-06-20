import { re } from "../util"

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

  /** Returns an iterator of matches against a word for this instance's pattern. */
  match(word: string) {
    return word.matchAll(this.pattern)
  }
}
