import { createID } from "@wikijump/util"
import type * as DF from "../definition"
import { Node } from "../node"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Wrapping } from "../types"
import { Rule } from "./rule"

export class State {
  declare id: number
  declare name: string
  declare node: Node
  declare begin: Rule
  declare end: Rule
  declare inside: (Rule | State)[] | null

  constructor(repo: Repository, id: number, state: DF.State) {
    let type = state.type ?? createID()
    let emit = state.type && state.emit !== false

    this.id = id
    this.name = type
    this.node = !emit ? Node.None : new Node(id, state)

    // prettier-ignore
    const begin = typeof state.begin === "string"
      ? repo.get(state.begin)
      : repo.add(state.begin)

    // prettier-ignore
    const end = typeof state.end === "string"
      ? repo.get(state.end)
      : repo.add(state.end)

    if (!(begin instanceof Rule) || !(end instanceof Rule)) {
      throw new Error(`Invalid state ${type}, rules not found or weren't rules`)
    }

    this.begin = begin
    this.end = end

    this.inside = null

    if (state.inside && state.inside !== "inherit") {
      this.inside = []
      for (const rule of state.inside) {
        // specifier for a rule
        if (typeof rule === "string") {
          const resolved = repo.get(rule)
          if (!(resolved instanceof Rule)) throw new Error(`Invalid inside rule`)
          this.inside.push(resolved)
        }
        // include
        else if ("include" in rule) {
          this.inside.push(...repo.include(rule.include))
        }
        // state
        else {
          this.inside.push(repo.add(rule))
        }
      }
    }
  }

  match(state: GrammarState, str: string, pos: number) {
    const matched = this.begin.match(state, str, pos)
    if (!matched) return null

    const inside = this.inside ? this.inside : state.stack.rules
    state.stack.push(this.node, inside, this.end)

    return matched.wrap(this.node, Wrapping.BEGIN)
  }
}
