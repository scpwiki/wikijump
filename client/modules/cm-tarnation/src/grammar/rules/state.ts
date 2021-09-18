import { createID } from "@wikijump/util"
import type * as DF from "../definition"
import { Node } from "../node"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Wrapping } from "../types"
import { Rule } from "./rule"

export class State {
  declare name: string
  declare node: Node
  declare begin: Rule
  declare end: Rule
  declare inside: (Rule | State)[] | null
  declare loose?: boolean

  constructor(repo: Repository, state: DF.State) {
    let type = state.type ?? createID()
    let emit = state.type && state.emit !== false

    this.name = type
    this.node = !emit ? Node.None : new Node(repo.id(), state)

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

    // loose mode
    if (state.inside === "loose") this.loose = true
    // list of rules or states
    else if (state.inside && typeof state.inside !== "string") {
      this.inside = repo.inside(state.inside)
    }
    // special case: state is supposed to nest, but no inside rules were given
    else if (!state.inside && state.nest) {
      this.inside = []
    }
  }

  match(state: GrammarState, str: string, pos: number) {
    // loose mode, doesn't actually affect the stack
    if (this.loose) {
      const endMatched = this.end.match(state, str, pos)
      if (endMatched) return endMatched.wrap(this.node, Wrapping.END)

      const beginMatched = this.begin.match(state, str, pos)
      if (beginMatched) return beginMatched.wrap(this.node, Wrapping.BEGIN)

      return null
    } else {
      const matched = this.begin.match(state, str, pos)
      if (!matched) return null

      const inside = this.inside ? this.inside : state.stack.rules
      state.stack.push(this.node, inside, this.end)

      return matched.wrap(this.node, Wrapping.BEGIN)
    }
  }
}
