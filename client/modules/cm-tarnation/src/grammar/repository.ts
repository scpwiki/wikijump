import type { VariableTable } from "../types"
import type * as DF from "./definition"
import type { Grammar } from "./grammar"
import { Node } from "./node"
import { Chain } from "./rules/chain"
import { LookupRule } from "./rules/lookup"
import { PatternRule } from "./rules/pattern"
import { Rule } from "./rules/rule"
import { State } from "./rules/state"

/** Holds the rules, states, etc. for a {@link Grammar}. */
export class Repository {
  /** Map of names to objects stored in this repository. */
  private map = new Map<string, Node | Rule | State>()

  /** Current {@link Node} ID. */
  private curID = 2 // starts at 2, because 0-1 are reserved

  constructor(
    public grammar: Grammar,
    public variables: VariableTable,
    public ignoreCase = false
  ) {}

  /** Returns every {@link Node} in the repository, sorted by ID. */
  nodes() {
    // deduplicates, runs the iterator
    const set = new Set(this.map.values())
    // get every node, remove any entries that are Node.None
    return Array.from(set)
      .map(v => (v instanceof Node ? v : v.node))
      .filter(v => v !== Node.None)
      .sort((a, b) => a.id - b.id)
  }

  /** Returns a fresh ID for use by a {@link Node}. */
  id() {
    const id = this.curID
    this.curID++
    return id
  }

  // repetitive signatures are due to how TypeScript handles overloading
  // it's a bit wacky, but it types nicely

  /**
   * Adds an object to the repository.
   *
   * @param obj - The object to add.
   * @param name - If given, this name will be assumed to be the object's
   *   name if it doesn't have one already.
   */
  add(obj: DF.State, name?: string): State
  add(obj: DF.Regex | DF.Rule, name?: string): Rule
  add(obj: DF.Regex | DF.Rule | DF.State, name?: string): Rule | State
  add(obj: DF.Node | DF.ReuseNode, name?: string): Node
  add(obj: DF.RepositoryItem, name?: string): Node | Rule | State
  add(obj: DF.RepositoryItem, name?: string): Node | Rule | State {
    // match pattern shorthand
    if (typeof obj === "string") {
      if (!name) throw new Error("name is required for shorthands")
      const pattern: DF.Pattern = { type: name, emit: false, match: obj }
      return this.add(pattern, name)
    }

    // node open bracket shorthand
    if ("open" in obj) {
      const node: DF.Node = {
        ...obj,
        type: `${obj.open}Open`,
        closedBy: `${obj.open}Close`
      }
      // @ts-ignore
      delete node.open
      return this.add(node, name)
    }

    // node close bracket shorthand
    if ("close" in obj) {
      const node: DF.Node = {
        ...obj,
        type: `${obj.close}Close`,
        openedBy: `${obj.close}Open`
      }
      // @ts-ignore
      delete node.close
      return this.add(node, name)
    }

    // reused node
    if ("is" in obj) {
      const result = this.get(obj.is)!
      if (!result) throw new Error(`reused node ${obj.is} not found`)
      return "node" in result ? result.node : result
    }

    // add name to node if it doesn't have one explicitly
    if (!obj.type && name) obj.type = name

    // prevents duplication when doing things out of order
    if (obj.type && this.map.get(obj.type)) {
      return this.map.get(obj.type)!
    }

    // lookup
    if ("lookup" in obj) {
      const lookup = new LookupRule(this, obj)
      this.map.set(lookup.name, lookup)
      return lookup
    }

    // pattern
    if ("match" in obj) {
      const pattern = new PatternRule(this, obj)
      this.map.set(pattern.name, pattern)
      return pattern
    }

    // chain
    if ("chain" in obj) {
      const chain = new Chain(this, obj)
      this.map.set(chain.name, chain)
      return chain
    }

    // state
    if ("begin" in obj) {
      const state = new State(this, obj)
      this.map.set(state.name, state)
      return state
    }

    // must be a node
    const node = new Node(this.id(), obj)
    this.map.set(node.name, node)
    return node
  }

  /**
   * Gets an object from this repository. If it doesn't exist already, the
   * grammar definition will be checked. Returns `undefined` if nothing can
   * be found.
   *
   * @param key - The name of the object to get.
   */
  get(key: string) {
    const result = this.map.get(key)

    // add missing item if possible
    if (!result) {
      if (this.grammar.def.repository?.[key]) {
        return this.add(this.grammar.def.repository[key], key)
      }
    }

    return result
  }

  /**
   * Processes an `include` from the grammar definition, by name.
   *
   * @param str - The name of the `include` to process.
   */
  include(str: string) {
    if (!this.grammar.def.includes) throw new Error("no includes defined")
    if (!this.grammar.def.includes[str]) throw new Error(`include ${str} not found`)
    return this.grammar.def.includes[str]
      .map(name => this.get(name)!)
      .filter(rule => !(rule instanceof Node)) as (Rule | State)[]
  }

  /**
   * Processes an "inside" list of rules/states/includes, returning a resolved list.
   *
   * @param rules - The list of rules/states/includes to process.
   */
  inside(rules: DF.Inside) {
    const inside = []
    for (const rule of rules) {
      // specifier for a rule
      if (typeof rule === "string") {
        const resolved = this.get(rule)
        if (!(resolved instanceof Rule) && !(resolved instanceof State)) {
          throw new Error(`Invalid inside rule`)
        }
        inside.push(resolved)
      }
      // include
      else if ("include" in rule) {
        inside.push(...this.include(rule.include))
      }
      // state or rule
      else {
        inside.push(this.add(rule))
      }
    }
    return inside
  }
}
