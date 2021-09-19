import type * as DF from "../definition"
import { isRegExpString } from "../helpers"
import { RegExpMatcher } from "../matchers/regexp"
import { StringMatcher } from "../matchers/string"
import type { Repository } from "../repository"
import { MatchOutput } from "../types"
import { Rule } from "./rule"

export class PatternRule extends Rule {
  private declare patterns: Arrayable<RegExpMatcher | StringMatcher>

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

      this.exec = this.execPatterns.bind(this)
    } else {
      this.patterns = isRegExpString(rule.match)
        ? new RegExpMatcher(rule.match, repo.ignoreCase, repo.variables)
        : new StringMatcher(rule.match, repo.ignoreCase, repo.variables)

      // directly bind to matcher for performance
      this.exec = this.patterns.match.bind(this.patterns)
    }
  }

  private execPatterns(str: string, pos: number) {
    const patterns = this.patterns as (RegExpMatcher | StringMatcher)[]
    for (let i = 0; i < patterns.length; i++) {
      const result = patterns[i].match(str, pos)
      if (result) return result
    }
    return null
  }
}
