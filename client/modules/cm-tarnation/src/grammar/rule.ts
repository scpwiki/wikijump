import { Action } from "./action"
import type * as DF from "./definition"
import type * as DM from "./demangler"
import type { Grammar, GrammarContext } from "./grammar"
import { Matcher } from "./matcher"

export class Rule {
  declare log?: string

  private declare target?: DF.SubRuleTarget
  private declare matcher: Matcher | "@DEFAULT"
  private declare action: Action

  constructor(grammar: Grammar, public id: number, rule: DM.Rule) {
    const { target, match, action } = rule

    if (!match) throw new Error("Rule must have a Match!")

    if (target) this.target = target

    if (match === "@DEFAULT") this.matcher = "@DEFAULT"
    else this.matcher = new Matcher(grammar, match)

    this.action = new Action(grammar, action)
    if (this.action.log) this.log = this.action.log
  }

  exec(cx: GrammarContext, str: string, pos: number) {
    // always match entire str past pos
    if (this.matcher === "@DEFAULT") {
      return this.action.exec(cx, [str.slice(pos)], pos)
    }

    const { target, last } = cx

    cx.target = this.target
    const found = this.matcher.exec(cx, str, pos)
    cx.target = target
    if (!found) return null

    if (this.target) {
      if (!cx.last) throw new Error("Rule target specified, but no last match!")
      return this.action.exec(cx, [str.slice(pos)], pos)
    } else {
      if (!cx.last || !cx.substate) cx.last = found
      const result = this.action.exec(cx, found, pos)
      cx.last = last
      return result
    }
  }
}
