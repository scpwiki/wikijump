import iterate from "iterare"
import { intersect, product, re } from "../util"
import type { Aff, Flags, FlagSet } from "./index"

// TODO: personally don't like how Spylls did this
// make my own solution that actually parses through a flagset step by step
// shouldn't be too hard, the syntax is relatively simple

/**
 * A `RegExp`-like rule for generating compound rules. It is an alternative
 * way of specifying compound words to the {@link Aff.COMPOUNDFLAG} (and
 * similar) {@link Flag}s. It uses the following syntax:
 *
 * ```text
 * COMPOUNDRULE A*B?CD
 * ```
 *
 * Which should be parsed as: A compound word may consist of zero or more
 * words with the {@link Flag} `A`, then optionally a word with the
 * {@link Flag} `B`, and then finally the compound must end with a word with
 * the {@link Flag} `C` and a word with the {@link Flag} `D`.
 *
 * The similarity of this to a `RegExp` is exploited by both Spylls and
 * Espells. The algorithm used to check for matches involves taking a
 * {@link FlagSet} (representing words) and turning it into a string that is
 * checked by a `RegExp`.
 */
export class CompoundRule {
  /** The {@link Flags} this rule is relevant to. */
  declare flags: Flags

  /** The `RegExp` used to check if a transformed {@link Flags} string is valid. */
  declare regex: RegExp

  /**
   * A fairly mangled looking `RegExp` that is used to determine if a
   * transformed {@link Flags} string is at least *partially* valid. This is
   * so that a compound word can be checked for if it *can* continue in some way.
   */
  declare partialRegex: RegExp

  /**
   * @param rule - The `RegExp`-like syntax to generate this rule.
   * @param aff - The {@link Aff} data to use when parsing flags.
   */
  constructor(rule: string, aff: Aff) {
    let parts: string[]
    if (rule.includes("(")) {
      this.flags = aff.parseFlags(/\((.+?)\)/.exec(rule)?.slice(1) ?? "")
      parts = /\([^*?]+?\)[*?]?/.exec(rule)?.slice(1) ?? []
    } else {
      this.flags = aff.parseFlags(rule.replaceAll(/[\*\?]/g, ""))
      parts = iterate(rule.matchAll(/[^*?][*?]?/g))
        .map(part => part[0].replaceAll(")", "\\)"))
        .toArray()
    }

    this.regex = re`/^${parts.join("")}$/`
    this.partialRegex = re`/^${parts.reduceRight((acc, cur) => `${cur}(${acc})?`)}$/`
  }

  /**
   * Determines if a {@link FlagSet} matches this rule.
   *
   * @param flags - The {@link FlagSet} to check.
   * @param partial - If true, the {@link FlagSet} will only need to
   *   partially match the rule to be considered valid. This is so that a
   *   compound word can be checked for if it *can* continue in some way.
   */
  match(flags: FlagSet, partial = false) {
    return iterate(flags)
      .map(f => product(...intersect(this.flags, f)))
      .some(fc => {
        const joined = iterate(fc).join("")
        return partial ? this.partialRegex.test(joined) : this.regex.test(joined)
      })
  }
}
