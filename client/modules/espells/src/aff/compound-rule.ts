import iterate from "iterare"
import { intersect, product, re } from "../util"
import type { Aff, Flags, FlagSet } from "./index"
import type { Flag } from "./types"

// TODO: make this not use a weird regex matching system
// I couldn't exactly wrap my brain around how to avoid using regex here
// I'm sure it's not hard. but right now this system is very slow because
// of all the permutations made by the `product(...relevant)` call.

/** {@link Rule} quantifiers. */
enum Quantifier {
  /** Must match the flag. */
  ONE,
  /** May optionally match the flag repeatedly. ("*" quantifier) */
  ZERO_OR_MORE,
  /** May optionally match the flag. ("?" quantifier) */
  ZERO_OR_ONE
}

/** A {@link CompoundRule} internal rule or step. */
interface Rule {
  /** The flag that needs to be matched. */
  flag: Flag
  /** The {@link Quantifier} for this rule. */
  quantifier: Quantifier
}

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

  /** The orderd list of parsed rules. */
  declare rules: Rule[]

  /**
   * @param rule - The `RegExp`-like syntax to generate this rule.
   * @param aff - The {@link Aff} data to use when parsing flags.
   */
  constructor(rule: string, aff: Aff) {
    this.flags = new Set()
    this.rules = parseCompoundRule(rule, aff)

    const parts: string[] = []

    for (const rule of this.rules) {
      this.flags.add(rule.flag)
      parts.push(rule.flag + quantifierChar(rule.quantifier))
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
    // empty rule set
    if (!this.rules.length) return false

    const relevant = iterate(flags)
      .map(f => intersect(this.flags, f))
      .filter(set => set.size !== 0)
      .toSet()

    // no flags in common
    if (relevant.size === 0) return false

    return iterate(product(...relevant)).some(fc =>
      partial ? this.partialRegex.test(fc.join("")) : this.regex.test(fc.join(""))
    )
  }
}

/**
 * Parses the source for a {@link CompoundRule}.
 *
 * @param text - The rule to parse.
 * @param aff - The {@link Aff} used for flag parsing.
 */
function parseCompoundRule(text: string, aff: Aff) {
  const rules: Rule[] = []

  for (let i = 0; i < text.length; i++) {
    let flag = ""
    let quantifier = Quantifier.ONE

    if (text[i] === "(") {
      while (text[i] !== ")" && i < text.length) {
        i++
        flag += text[i]
      }

      if (text[i + 1] === "?") {
        quantifier = Quantifier.ZERO_OR_ONE
        i++
      } else if (text[i + 1] === "*") {
        quantifier = Quantifier.ZERO_OR_ONE
        i++
      }

      rules.push({ flag, quantifier })

      continue
    }

    switch (aff.FLAG) {
      case "UTF-8":
      case "short": {
        flag += text[i]
        break
      }
      case "long": {
        flag += text.slice(i, i + 2)
        break
      }
      case "numeric": {
        while (/\d/.test(text[i])) {
          flag += text[i]
          i++
        }
        i-- // move back a position to make the code ahead consistent
      }
    }

    if (text[i + 1] === "?") {
      quantifier = Quantifier.ZERO_OR_ONE
      i++
    } else if (text[i + 1] === "*") {
      quantifier = Quantifier.ZERO_OR_MORE
      i++
    }

    rules.push({ flag, quantifier })
  }

  return rules
}

/** Returns the relevant character used to denote a {@link Quantifier}. */
function quantifierChar(quantifier: Quantifier) {
  // prettier-ignore
  switch (quantifier) {
    case Quantifier.ZERO_OR_MORE: return "*"
    case Quantifier.ZERO_OR_ONE: return "?"
  }
  return ""
}
