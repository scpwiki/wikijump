import { createID } from "@wikijump/util"
import { Wrapping } from "../../enums"
import type * as DF from "../definition"
import { Node } from "../node"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

/**
 * A sort of {@link Rule}-like object that affects a {@link GrammarStack}. It
 * has `begin` and `end` properties which are {@link Rule}s that are used
 * for switching states.
 */
export class State {
  /** The name of this state. */
  declare name: string

  /** The {@link Node} this state wraps tokens with. */
  declare node: Node

  /** The {@link Rule} that starts this state. */
  declare begin: Rule

  /** The {@link Rule} that ends this state. */
  declare end: Rule

  /** The list of rules/states to loop parsing with when in this state. */
  declare inside: (Rule | State)[] | null

  /**
   * If true, this state won't affect the stack, but instead manipulate the
   * parser to "loosely" wrap tokens.
   */
  declare loose?: boolean

  /**
   * @param repo - The {@link Repository} to add this state to.
   * @param state - The definition for this state.
   */
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

  /**
   * @param state - The current {@link GrammarState}.
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
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
