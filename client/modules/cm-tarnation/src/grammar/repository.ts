import type * as DF from "./definition"
import type { Grammar } from "./grammar"
import { Node } from "./node"
import { LookupRule } from "./rules/lookup"
import { PatternRule } from "./rules/pattern"
import { Rule } from "./rules/rule"
import { State } from "./rules/state"
import type { VariableTable } from "./types"

export class Repository {
  private map = new Map<string, Node | Rule | State>()
  private curID = 3 // starts at 2, because 0-2 are reserved

  constructor(
    public grammar: Grammar,
    public variables: VariableTable,
    public ignoreCase = false,
    public includes: Record<string, string[]> = {}
  ) {}

  nodes() {
    // deduplicates, runs the iterator
    const set = new Set(this.map.values())
    // get every node, remove any entries that are Node.None
    return Array.from(set)
      .map(v => (v instanceof Node ? v : v.node))
      .filter(v => v !== Node.None)
      .sort((a, b) => a.id - b.id)
  }

  id() {
    const id = this.curID
    this.curID++
    return id
  }

  // repetitive signatures are due to how TypeScript handles overloading
  // it's a bit wacky, but it types nicely

  add(state: DF.State, name?: string): State
  add(rule: DF.Regex | DF.Rule, name?: string): Rule
  add(rule: DF.Regex | DF.Rule | DF.State, name?: string): Rule | State
  add(node: DF.Node | DF.ReuseNode, name?: string): Node
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

    // make sure we have a name/type for the node
    if (!("is" in obj) && !("template" in obj)) {
      if (name && !obj.type) obj = { ...obj, type: name }
    }

    // reused node
    if ("is" in obj) return this.get(obj.is)!

    // prevents duplication when doing things out of order
    if ((obj.type && this.map.get(obj.type)) || (name && this.map.get(name))) {
      return this.map.get(obj.type! || name!)!
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
      this.variables[pattern.name] = obj.match
      return pattern
    }

    // chain
    if ("chain" in obj) {
      // TODO: chain
      throw new Error("not implemented")
    }

    // state
    if ("begin" in obj) {
      const state = new State(this, obj)
      this.map.set(state.name, state)
      return state
    }

    // must be a node
    else {
      const id = this.id()
      const node = new Node(id, obj)
      this.map.set(node.name, node)
      return node
    }
  }

  get(key: string) {
    const result = this.map.get(key)

    // add missing item if possible
    if (!result) {
      if (this.grammar.def.repository?.[key]) {
        return this.add(this.grammar.def.repository[key])
      }
    }

    return result
  }

  include(str: string) {
    if (!this.includes[str]) throw new Error(`include ${str} not found`)
    return this.includes[str]
      .map(name => this.get(name)!)
      .filter(rule => !(rule instanceof Node)) as (Rule | State)[]
  }

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
