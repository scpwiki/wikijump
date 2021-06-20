import { CONSTANTS as C } from "../constants"
import { re } from "../util"

// TODO: use homegrown metaphone implementation (maybe)

/**
 * A table for metaphone transformations. These transformations may provide
 * superior suggestions because they describe similar sounding (as in
 * spoken) syllables, which will be used by the suggestion engine to find
 * words that may not be spelled similarly but *sound* similar.
 *
 * It roughly uses the following syntax:
 *
 * ```text
 * PHONE <number of entries>
 * PHONE <pattern> <replacement>
 * ```
 *
 * `replacement` is a simple string, with the special value `_`
 * (underscore) meaning an empty string. `pattern` is complex, and
 * currently isn't very well documented. Additionally, both Spylls and
 * Espells do not fully implement Hunspell's more intricate details, such
 * as rule prioritizing and concepts like "follow-up rules".
 *
 * Dictionaries that use this feature are unfortunately quite rare.
 */
export class PhonetTable {
  rules: Record<string, PhonetTableRule[]> = {}

  constructor(table: [string, string][]) {
    for (const [search, replacement] of table) {
      const match = C.PHONET_RULE_REGEX.exec(search)
      if (!match) throw new SyntaxError(`Invalid PhonetTable pattern '${search}'`)

      const [letters, optional, lookahead, flags, priority] = match

      let text = [...letters]
      if (optional) text.push(`[${optional}]`)

      let regex: RegExp
      if (lookahead) {
        const la = lookahead.length
        regex = re`/${text.slice(0, -la).join("")}(?=${text.slice(-la).join("")})/`
      } else {
        regex = re`/${text.join("")}/`
      }

      this.rules[search[0]] ??= []
      this.rules[search[0]].push(
        new PhonetTableRule(
          regex,
          replacement,
          flags?.includes("^"),
          flags?.includes("$"),
          Boolean(lookahead),
          priority ? parseInt(priority) : 5
        )
      )
    }
  }
}

/**
 * An individual phonetic table rule.
 *
 * @see {@link PhonetTable}
 */
class PhonetTableRule {
  constructor(
    /** The `RegExp` used for checking if this rule applies. */
    public search: RegExp,
    /** The string to represent a matched phoneme with. */
    public replacement: string,
    /** If true, this rule only applies at the start of a word. */
    public start = false,
    /** If true, this rule only applies at the end of a word. */
    public end = false,
    /** Currently unusued in both Spylls and Espells. */
    public followup = true,
    /** Currently unusued in both Spylls and Espells. */
    public priority = 5
  ) {}

  /**
   * Checks if a rule is matched by this rule, and if it is, returns the
   * `RegExpExecArray` match, otherwise returning false.
   *
   * @param word - The word to check.
   * @param pos - The position in the word to check.
   */
  match(word: string, pos: number) {
    if (this.start && pos > 0) return false
    this.search.lastIndex = pos
    const match = this.search.exec(word)
    if (match) {
      if (this.end) {
        return match[0].length !== word.length ? false : match
      } else {
        return match
      }
    }
    return false
  }
}
