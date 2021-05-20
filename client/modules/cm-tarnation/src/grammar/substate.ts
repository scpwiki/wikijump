import { isString } from "is-what"
import type * as DM from "./demangler"
import type { Grammar, GrammarContext } from "./grammar"
import type { Matched } from "./matched"

export class SubState {
  declare repeat: boolean
  declare optional: boolean
  declare all: boolean
  declare strict: boolean

  constructor(
    public grammar: Grammar,
    { strict = true, repeat, optional, all, rules }: DM.Action
  ) {
    if (!rules) throw new Error("Substate must have rules!")

    this.repeat = strict ? false : repeat ?? true
    this.optional = strict ? false : optional ?? true
    this.all = strict ? true : all ?? false
    this.strict = strict

    if (isString(rules)) grammar.addSubstate(this, grammar.addState(rules))
    else grammar.addSubstate(this, rules)
  }

  exec(cx: GrammarContext, str: string, pos: number): Matched[] | null {
    const lastSubstate = cx.substate
    const matches: Matched[] = []

    cx.substate = this

    let offset = 0
    let match: Matched | null = null
    // prettier-ignore
    while (offset < str.length &&
      ((match = this.grammar.match(cx, str, offset)) || (!this.all && this.repeat))
    ) {
      if (!match) {
        offset += 1
        continue
      }
      match.offset = pos + offset
      matches.push(match)
      offset += match.length
      if (!this.repeat) break
    }

    cx.substate = lastSubstate

    if (!this.optional && !matches.length) return null
    if (this.all && offset < str.length) return null
    return matches
  }
}
