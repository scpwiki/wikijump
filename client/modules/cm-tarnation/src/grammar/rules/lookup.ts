import type * as DF from "../definition"
import { LookupMatcher } from "../matchers/lookup"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

export class LookupRule extends Rule {
  private declare lookup: LookupMatcher

  constructor(repo: Repository, rule: DF.Lookup) {
    super(repo, rule)

    let strings = rule.lookup
    if (typeof strings === "string") {
      const [, ident] = strings.split(":")
      const value = repo.variables[ident]
      if (!Array.isArray(value)) throw new Error(`Variable ${ident} is not an array`)
      strings = value
    }

    this.lookup = new LookupMatcher(strings, repo.ignoreCase, repo.variables)
  }

  exec(state: GrammarState, str: string, pos: number) {
    return this.lookup.match(str, pos)
  }
}
