import { re } from "../util"

/** Instance of a `.aff` `REP` replacement pattern. */
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
