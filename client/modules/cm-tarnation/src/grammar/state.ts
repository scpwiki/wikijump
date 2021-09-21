import type { GrammarStackElement, MatchOutput, VariableTable } from "../types"
import type { Node } from "./node"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"

/** Internal state for a {@link Grammar}. */
export class GrammarState {
  /**
   * @param variables - The variables to use when substituting.
   * @param context - The current context table.
   * @param stack - The current {@link GrammarStack}.
   * @param last - The last {@link MatchOutput} that was matched.
   */
  constructor(
    public variables: VariableTable,
    public context: Record<string, string> = {},
    public stack: GrammarStack = new GrammarStack(),
    public last?: MatchOutput
  ) {}

  /**
   * Sets a key in the context table.
   *
   * @param key - The key to set.
   * @param value - The value to set. If `null`, the key will be removed.
   */
  set(key: string, value: string | null) {
    if (value === null) {
      this.context = { ...this.context }
      delete this.context[key]
    } else {
      const subbed = this.sub(value)
      if (typeof subbed !== "string") throw new Error("Invalid context value")
      this.context = { ...this.context, [key]: subbed }
    }
  }

  /**
   * Gets a key from the context table.
   *
   * @param key - The key to get.
   */
  get(key: string): string | null {
    return this.context[key] ?? null
  }

  /**
   * Expands any substitutions found in the given string.
   *
   * @param str - The string to expand.
   */
  sub(str: string) {
    if (str[0] !== "$") return str

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

  /**
   * Returns if another {@link GrammarState} is effectively equivalent to this one.
   *
   * @param other - The other {@link GrammarState} to compare to.
   */
  equals(other: GrammarState) {
    if (this.variables !== other.variables) return false
    if (!contextEquivalent(this.context, other.context)) return false
    if (!this.stack.equals(other.stack)) return false
    return true
  }

  /** Returns a new clone of this state, including its stack. */
  clone() {
    return new GrammarState(this.variables, this.context, this.stack.clone(), this.last)
  }
}

/** A stack of {@link GrammarStackElement}s used by a {@link Grammar}. */
export class GrammarStack {
  /** @param stack - The current stack. */
  constructor(public stack: GrammarStackElement[] = []) {}

  /**
   * Pushes a new {@link GrammarStackElement}.
   *
   * @param node - The parent {@link Node}.
   * @param rules - The rules to loop parsing with.
   * @param end - A specific {@link Rule} that, when matched, should pop
   *   this element off.
   */
  push(node: Node, rules: (Rule | State)[], end: Rule) {
    this.stack = [...this.stack, { node, rules, end }]
  }

  /** Pops the last element on the stack. */
  pop() {
    if (this.stack.length === 0) throw new Error("Grammar stack underflow")
    this.stack = this.stack.slice(0, -1)
  }

  /**
   * Remove every element at or beyond the index given.
   *
   * @param index - The index to remove elements at or beyond.
   */
  close(idx: number) {
    this.stack = this.stack.slice(0, idx)
  }

  /**
   * Returns if another {@link GrammarStack} is effectively equivalent to this one.
   *
   * @param equals - The other {@link GrammarStack} to compare to.
   */
  equals(other: GrammarStack) {
    if (this === other) return true
    if (this.length !== other.stack.length) return false
    for (let i = 0; i < this.stack.length; i++) {
      if (!stackElementEquivalent(this.stack[i], other.stack[i])) return false
    }
    return true
  }

  /** The number of elements on the stack. */
  get length() {
    return this.stack.length
  }

  /** The last parent {@link Node}. */
  get node() {
    return this.stack[this.stack.length - 1].node
  }

  /** The last list of rules. */
  get rules() {
    return this.stack[this.stack.length - 1].rules
  }

  /** The last end rule. */
  get end() {
    return this.stack[this.stack.length - 1].end
  }

  /** Returns a new clone of this stack. */
  clone() {
    return new GrammarStack(this.stack)
  }
}

function stackElementEquivalent(a: GrammarStackElement, b: GrammarStackElement) {
  // do quick checks first
  if (a.node !== b.node || a.end !== b.end || a.rules.length !== b.rules.length) {
    return false
  }
  for (let i = 0; i < a.rules.length; i++) {
    if (a.rules[i] !== b.rules[i]) return false
  }
  return true
}

function contextEquivalent(a: Record<string, string>, b: Record<string, string>) {
  if (a === b) return true
  if (Object.keys(a).length !== Object.keys(b).length) return false
  for (const key in a) {
    if (a[key] !== b[key]) return false
  }
  return true
}
