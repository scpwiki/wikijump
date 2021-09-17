import type { Node } from "./node"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"
import type { GrammarStackElement, MatchOutput, VariableTable } from "./types"

export class GrammarState {
  constructor(
    public variables: VariableTable,
    public context: Record<string, string> = {},
    public stack: GrammarStack = new GrammarStack(),
    public last?: MatchOutput
  ) {}

  set(key: string, value: string | null): void {
    if (value === null) {
      this.context = { ...this.context }
      delete this.context[key]
    } else {
      const subbed = this.sub(value)
      if (typeof subbed !== "string") throw new Error("Invalid context value")
      this.context = { ...this.context, [key]: subbed }
    }
  }

  get(key: string): string | null {
    return this.context[key] ?? null
  }

  sub(str: string) {
    if (!str.startsWith("$")) return str

    // variable substitution
    if (str.startsWith("$var:")) {
      const [, name] = str.split(":")
      return this.variables[name]
    }
    // context substitution
    else if (str.startsWith("$ctx:")) {
      const [, name] = str.split(":")
      return this.context[name]
    }
    // match/capture substition
    else if (this.last?.captures) {
      const [, index] = str.split("$")
      return this.last.captures[parseInt(index, 10)]
    }

    throw new Error("Couldn't resolve substitute")
  }

  adopt(state: GrammarState) {
    this.variables = state.variables
    this.context = state.context
    this.stack = state.stack
    this.last = state.last
  }

  clone() {
    return new GrammarState(this.variables, this.context, this.stack.clone(), this.last)
  }
}

export class GrammarStack {
  constructor(public stack: GrammarStackElement[] = []) {}

  push(node: Node, rules: (Rule | State)[], end: Rule) {
    this.stack = [...this.stack, { node, rules, end }]
  }

  pop() {
    if (this.stack.length === 0) throw new Error("Grammar stack underflow")
    this.stack = this.stack.slice(0, -1)
  }

  get node() {
    return this.stack[this.stack.length - 1].node
  }

  get rules() {
    return this.stack[this.stack.length - 1].rules
  }

  get end() {
    return this.stack[this.stack.length - 1].end
  }

  clone() {
    return new GrammarStack(this.stack)
  }
}
