import type * as DF from "../definition"
import { isRegExpString } from "../helpers"
import { RegExpMatcher } from "../matchers/regexp"
import { StringMatcher } from "../matchers/string"
import type { Repository } from "../repository"
import { MatchOutput } from "../types"
import { Rule } from "./rule"

/**
 * A {@link Rule} subclass that uses {@link RegExpMatcher} or
 * {@link StringMatcher} instances for the underlying pattern.
 */
export class PatternRule extends Rule {
  /**
   * A list of patterns to check. This is a list of alternatives, so if any
   * matches, the rule matches.
   */
  private declare patterns: (RegExpMatcher | StringMatcher)[]

  declare exec: (str: string, pos: number) => MatchOutput

  /**
   * @param repo - The {@link Repository} to add this rule to.
   * @param rule - The rule definition.
   */
  constructor(repo: Repository, rule: DF.Pattern) {
    super(repo, rule)

    // bit odd, but we're doing this so we can bind directly
    // to the matcher if possible. this requires some shenanigans
    if (Array.isArray(rule.match)) {
      this.patterns = rule.match.map(pattern => {
        return isRegExpString(pattern)
          ? new RegExpMatcher(pattern, repo.ignoreCase, repo.variables)
          : new StringMatcher(pattern, repo.ignoreCase, repo.variables)
      })
      // eslint-disable-next-line @typescript-eslint/unbound-method
      this.exec = this.execPatterns
    } else {
      const pattern = isRegExpString(rule.match)
        ? new RegExpMatcher(rule.match, repo.ignoreCase, repo.variables)
        : new StringMatcher(rule.match, repo.ignoreCase, repo.variables)

      // normally this could just be a bound function,
      // but I didn't see any significant performance benefit
      // versus just using a closure.
      this.exec = function (str: string, pos: number) {
        return pattern.match(str, pos)
      }
    }
  }

  /**
   * Internal function that is set as the `exec` method when the `patterns`
   * list is being used.
   */
  private execPatterns(str: string, pos: number) {
    for (let i = 0; i < this.patterns.length; i++) {
      const result = this.patterns[i].match(str, pos)
      if (result) return result
    }
    return null
  }
}
