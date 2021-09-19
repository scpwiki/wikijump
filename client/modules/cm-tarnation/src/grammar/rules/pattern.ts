import type * as DF from "../definition"
import { isRegExpString } from "../helpers"
import { RegExpMatcher } from "../matchers/regexp"
import { StringMatcher } from "../matchers/string"
import type { Repository } from "../repository"
import { MatchOutput } from "../types"
import { Rule } from "./rule"

export class PatternRule extends Rule {
  private declare patterns: (RegExpMatcher | StringMatcher)[]

  declare exec: (str: string, pos: number) => MatchOutput | null

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

  private execPatterns(str: string, pos: number) {
    for (let i = 0; i < this.patterns.length; i++) {
      const result = this.patterns[i].match(str, pos)
      if (result) return result
    }
    return null
  }
}
