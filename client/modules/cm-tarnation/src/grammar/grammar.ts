import type * as DF from "./definition"
import { re } from "./helpers"
import { Matched } from "./matched"
import { Node } from "./node"
import { Repository } from "./repository"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"
import { GrammarStack, GrammarState } from "./state"
import { VariableTable, Wrapping } from "./types"

export class Grammar {
  declare data: Record<string, any>
  declare repository: Repository
  declare root: (Rule | State)[]
  declare global?: (Rule | State)[]
  declare default?: Node

  constructor(public def: DF.Grammar, public variables: VariableTable = {}) {
    // process language data

    this.data = {}

    if (def.comments) this.data.commentTokens = def.comments
    if (def.closeBrackets) this.data.closeBrackets = def.closeBrackets
    if (def.wordChars) this.data.wordChars = def.wordChars
    if (def.indentOnInput) {
      const regex = re(def.indentOnInput)
      if (!regex) throw new Error(`Invalid indentOnInput: ${def.indentOnInput}`)
      this.data.indentOnInput = regex
    }

    // populate variable table with repository patterns
    if (def.repository) {
      for (const name in def.repository) {
        const value = def.repository[name]
        if (typeof value === "string") variables[name] = value
        else if ("match" in value) variables[name] = value.match
      }
    }

    // setup repository, add rules, etc.

    this.repository = new Repository(this, variables, def.ignoreCase, def.includes)

    if (def.default) this.default = this.repository.add(def.default)

    if (def.repository) {
      for (const name in def.repository) {
        this.repository.add(def.repository[name], name)
      }
    }

    this.root = this.repository.inside(def.root)

    if (def.global) this.global = this.repository.inside(def.global)
  }

  startState() {
    return new GrammarState(
      this.variables,
      {},
      new GrammarStack([{ node: Node.None, rules: this.root, end: null }])
    )
  }

  match(state: GrammarState, str: string, pos: number, offset = 0) {
    // check end node(s)
    if (state.stack.end) {
      for (let i = state.stack.length - 1; i > 0; i--) {
        const element = state.stack.get(i)
        if (element.end) {
          let result = element.end.match(state.clone(), str, pos)
          if (result) {
            // close every stack element above the one that matched
            // e.g. i = 5, stack.length = 7, close 5, 6
            for (let j = state.stack.length - 1; j >= i; j--) {
              result = result.wrap(result.state.stack.get(j).node, Wrapping.END)
            }
            result.state.stack.close(i)
            if (offset !== pos) result.offset(offset)
            return result
          }
        }
      }
    }

    // normal matching
    const rules = state.stack.rules
    for (let i = 0; i < rules.length; i++) {
      const rule = rules[i]
      const result = rule.match(state.clone(), str, pos)
      if (result) {
        if (offset !== pos) result.offset(offset)
        return result
      }
    }

    // global matching
    if (this.global) {
      for (let i = 0; i < this.global.length; i++) {
        const rule = this.global[i]
        const result = rule.match(state.clone(), str, pos)
        if (result) {
          if (offset !== pos) result.offset(offset)
          return result
        }
      }
    }

    if (this.default) {
      const result = new Matched(
        state.clone(),
        this.default,
        str.slice(pos, pos + 1),
        offset
      )
      if (result) return result
    }

    return null
  }

  matchAll(state: GrammarState, str: string, offset = 0) {
    const results: Matched[] = []

    let pos = 0
    while (pos < str.length) {
      const result = this.match(state, str, pos, offset)
      if (!result) pos++
      else {
        results.push(result)
        pos += result.length
      }
    }

    return results.length ? results : null
  }
}
