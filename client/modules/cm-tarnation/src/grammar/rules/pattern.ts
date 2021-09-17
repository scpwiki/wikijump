import type * as DF from "../definition"
import { isRegExpString } from "../helpers"
import { RegExpMatcher } from "../matchers/regexp"
import { StringMatcher } from "../matchers/string"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

export class PatternRule extends Rule {
  private declare patterns: (RegExpMatcher | StringMatcher)[]

  constructor(repo: Repository, id: number, rule: DF.Pattern) {
    super(repo, id, rule)

    const patterns = Array.isArray(rule.match) ? rule.match : [rule.match]
    this.patterns = patterns.map(pattern => {
      return isRegExpString(pattern)
        ? new RegExpMatcher(pattern, repo.ignoreCase, repo.variables)
        : new StringMatcher(pattern, repo.ignoreCase, repo.variables)
    })
  }

  exec(state: GrammarState, str: string, pos: number) {
    for (let i = 0; i < this.patterns.length; i++) {
      const result = this.patterns[i].match(str, pos)
      if (result) return result
    }
    return null
  }
}
