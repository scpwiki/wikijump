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
  declare inside: (Rule | State)[] | Node | null

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

    // states handle nesting differently, so we don't want
    // to use the normal nesting behavior
    if (state.nest) {
      const nest = state.nest
      delete state.nest // don't pass to node
      this.inside = new Node(repo.id(), { type: `${type}_Nest${nest}`, nest })
    }

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

    if (!this.inside) {
      if (state.inside) {
        if (state.inside === "loose") this.loose = true
        else if (state.inside === "inherit") this.inside = null
        else if (!Array.isArray(state.inside)) this.inside = repo.add(state.inside)
        else this.inside = repo.inside(state.inside)
      } else {
        this.inside = null
      }
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
      let matched = this.begin.match(state, str, pos)
      if (!matched) return null
      matched = matched.wrap(this.node, Wrapping.BEGIN)

      if (this.inside instanceof Node) {
        matched.push(this.inside, 1)
        state.stack.push(this.node, [], this)
      } else {
        const inside = this.inside ? this.inside : state.stack.rules
        state.stack.push(this.node, inside, this)
      }

      return matched
    }
  }

  /**
   * @param state - The current {@link GrammarState}.
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  close(state: GrammarState, str: string, pos: number) {
    if (this.loose) throw new Error("Closing a loose state is not supported")

    let matched = this.end.match(state, str, pos)
    if (!matched) return null

    matched = matched.wrap(this.node, Wrapping.END)
    if (this.inside instanceof Node) matched.push(this.inside, -1)
    matched.state.stack.pop()

    return matched
  }
}
