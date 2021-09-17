import ftmlYAML from "../../../cm-lang-ftml/src/grammars/ftml.yaml"
import type * as DF from "./definition"
import { Matched } from "./matched"
import type { Node } from "./node"
import { Repository } from "./repository"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"
import { GrammarStack, GrammarState } from "./state"
import { VariableTable, Wrapping } from "./types"

export class Grammar {
  declare repository: Repository
  declare rootNode: Node
  declare root: (Rule | State)[]
  declare global?: (Rule | State)[]
  declare default?: Node

  constructor(def: DF.Grammar, public variables: VariableTable = {}) {
    this.repository = new Repository(variables, def.ignoreCase, def.includes)

    this.rootNode = this.repository.add({ type: "Root" })

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
      new GrammarStack([{ node: this.rootNode, rules: this.root, end: null }])
    )
  }

  match(state: GrammarState, str: string, pos: number) {
    // normal matching
    const rules = state.stack.rules
    for (let i = 0; i < rules.length; i++) {
      const rule = rules[i]
      const result = rule.match(state.clone(), str, pos)
      if (result) return result
    }

    // check end node
    if (state.stack.end) {
      const result = state.stack.end.match(state.clone(), str, pos)
      if (result) {
        const wrapped = result.wrap(state.stack.node, Wrapping.END)
        wrapped.state.stack.pop()
        return wrapped
      }
    }

    // global matching
    if (this.global) {
      for (let i = 0; i < this.global.length; i++) {
        const rule = this.global[i]
        const result = rule.match(state.clone(), str, pos)
        if (result) return result
      }
    }

    if (this.default) {
      const result = new Matched(state, this.default, str.slice(pos, pos + 1), pos)
      if (result) return result
    }

    return null
  }
}

const asdf2 = new Grammar(ftmlYAML as any)

console.log(asdf2)
console.log(asdf2.startState())
