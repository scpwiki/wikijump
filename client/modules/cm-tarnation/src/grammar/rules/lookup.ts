import type { MatchOutput } from "../../types"
import type * as DF from "../definition"
import { LookupMatcher } from "../matchers/lookup"
import type { Repository } from "../repository"
import { Rule } from "./rule"

/**
 * A {@link Rule} subclass that uses a {@link LookupMatcher} for the
 * underlying pattern. This is a list of string options converted into a
 * very quick to match list internally.
 */
export class LookupRule extends Rule {
  /** The internal {@link LookupMatcher}. */
  private declare lookup: LookupMatcher

  declare exec: (str: string, pos: number) => MatchOutput

  /**
   * @param repo - The {@link Repository} to add this rule to.
   * @param rule - The rule definition.
   */
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
    this.exec = this.lookup.match.bind(this.lookup)
  }
}
