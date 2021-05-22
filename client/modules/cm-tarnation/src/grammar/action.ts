import { has, hasSigil, removeUndefined } from "wj-util"
import type * as DF from "./definition"
import type * as DM from "./demangler"
import { getMatchState, Grammar, GrammarContext, ParserAction } from "./grammar"
import { Matched } from "./matched"
import { SubState } from "./substate"

export enum ActionMode {
  Normal,
  SubState,
  Group,
  Bracket,
  Rematch
}

export class Action {
  declare type: string
  declare group?: Action[]
  declare next?: string
  declare log?: string
  declare switchTo?: string
  declare open?: ParserAction
  declare close?: ParserAction
  declare embedded?: string
  declare context?: DF.Context

  private declare substate?: SubState

  declare mode: ActionMode

  constructor(private grammar: Grammar, action?: DM.Action) {
    this.type = ""

    if (action) {
      const { type, next, open, close, log, switchTo, embedded, context } = action
      Object.assign(
        this,
        removeUndefined({ type, next, open, close, log, switchTo, embedded, context })
      )
      if (has("rules", action)) this.substate = new SubState(grammar, action)
      else if (action.group) {
        this.group = action.group.map(act => new Action(grammar, act))
      }
    }

    if (this.substate) this.mode = ActionMode.SubState
    else if (hasSigil(this.type, "@RE")) this.mode = ActionMode.Rematch
    else if (hasSigil(this.type, "@BR")) this.mode = ActionMode.Bracket
    else if (this.group) this.mode = ActionMode.Group
    else this.mode = ActionMode.Normal

    // add types
    if (this.type && !hasSigil(this.type, "@")) grammar.types.add(this.type)
    if (this.open) for (const [type] of this.open) grammar.types.add(type)
    if (this.close) for (const [type] of this.close) grammar.types.add(type)
  }

  private updateContext(cx: GrammarContext) {
    if (!this.context) return cx.context
    const added: DF.Context = {}
    for (const key in this.context) {
      const prop = this.context[key]
      if (prop) added[key] = Grammar.sub(cx, prop)
    }
    return (cx.context = { ...cx.context, ...added })
  }

  exec(cx: GrammarContext, found: string[], pos: number): Matched | null {
    const matched = new Matched(found[0], this, pos)

    switch (this.mode) {
      case ActionMode.Normal: {
        matched.context = this.updateContext(cx)
        matched.state = getMatchState(cx)
        return matched
      }

      case ActionMode.Bracket: {
        matched.context = this.updateContext(cx)
        const state = getMatchState(cx)
        matched.state = state
        const action = this.grammar.findBracket(found[0], this.type)
        if (action) {
          return Matched.extend(matched, [new Matched(found[0], action, pos, state)])
        }
        return null
      }

      case ActionMode.Rematch: {
        const context = this.updateContext(cx)
        const state = getMatchState(cx)
        return new Matched("", this, pos, state, context)
      }

      case ActionMode.SubState: {
        const sub = this.substate!.exec(cx, found[0], pos)
        if (!sub) return null
        matched.context = this.updateContext(cx)
        matched.state = getMatchState(cx)
        return Matched.extend(matched, sub)
      }

      case ActionMode.Group: {
        const [, ...captures] = found
        let offset = pos
        const group: (Matched | null)[] = []
        for (let i = 0; i < captures.length; i++) {
          const capture = captures[i]
          const action = this.group![i]
          if (!action) throw new Error("Disjointed match group count!")
          if (!capture) continue
          group.push(action.exec(cx, [capture], offset))
          offset += capture.length
        }
        matched.context = this.updateContext(cx)
        matched.state = getMatchState(cx)
        return Matched.extend(matched, group)
      }

      default: {
        return null
      }
    }
  }
}
