import { re } from "../util"
import type * as DF from "./definition"
import { Matched } from "./matched"
import { Node } from "./node"
import { Repository } from "./repository"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"
import { GrammarStack, GrammarState } from "./state"
import { VariableTable, Wrapping } from "./types"

const EAGER_RECOVERY = false

/** Grammar/dumb-tokenizer for a {@link TarnationLanguage}. */
export class Grammar {
  /** Extra language data props, as parsed from the definition. */
  declare data: Record<string, any>

  /** {@link Repository} of rules, states, and nodes used by this grammar. */
  declare repository: Repository

  /** The root state/rules to begin tokenizing with. */
  declare root: (Rule | State)[]

  /** Fallback rules which are checked if nothing else did. */
  declare global?: (Rule | State)[]

  /** Default {@link Node} that is returned if nothing matched. */
  declare default?: Node

  /**
   * @param def - The definition grammar to compile.
   * @param variables - {@link Variable}s to pass to the compiled grammar.
   */
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

    this.repository = new Repository(this, variables, def.ignoreCase)

    if (def.default) this.default = this.repository.add(def.default)

    if (def.repository) {
      for (const name in def.repository) {
        this.repository.add(def.repository[name], name)
      }
    }

    this.root = this.repository.inside(def.root)

    if (def.global) this.global = this.repository.inside(def.global)
  }

  /** Returns a {@link GrammarState} setup for this grammar's default state. */
  startState() {
    return new GrammarState(
      this.variables,
      {},
      new GrammarStack([{ node: Node.None, rules: this.root, end: null }])
    )
  }

  /**
   * Runs a match against a string (starting from a given position).
   *
   * @param state - The {@link GrammarState} to run the match with.
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   * @param offset - The offset to apply to the resulting {@link Matched}'s
   *   `from` position.
   */
  match(state: GrammarState, str: string, pos: number, offset = 0) {
    // check end node(s)
    if (state.stack.end) {
      // disabled for now, because it fails in a few scenarios
      // however it's still potentially useful so I'll keep it here for now
      if (EAGER_RECOVERY) {
        for (let i = state.stack.length - 1; i > 0; i--) {
          const element = state.stack.stack[i]
          if (element.end) {
            let result = element.end.match(state, str, pos)
            if (result) {
              result = result.wrap(element.node, Wrapping.END)
              result.state.stack.close(i)
              if (offset !== pos) result.offset(offset)
              return result
            }
          }
        }
      } else {
        let result = state.stack.end.match(state, str, pos)
        if (result) {
          result = result.wrap(state.stack.node, Wrapping.END)
          result.state.stack.pop()
          if (offset !== pos) result.offset(offset)
          return result
        }
      }
    }

    // normal matching
    const rules = state.stack.rules
    for (let i = 0; i < rules.length; i++) {
      const rule = rules[i]
      const result = rule.match(state, str, pos)
      if (result) {
        if (offset !== pos) result.offset(offset)
        return result
      }
    }

    // global matching
    if (this.global) {
      for (let i = 0; i < this.global.length; i++) {
        const rule = this.global[i]
        const result = rule.match(state, str, pos)
        if (result) {
          if (offset !== pos) result.offset(offset)
          return result
        }
      }
    }

    if (this.default) {
      const result = new Matched(state, this.default, str.slice(pos, pos + 1), offset)
      if (result) return result
    }

    return null
  }
}
