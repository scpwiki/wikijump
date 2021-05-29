import { klona } from "klona"
import type { ParserElementStack } from "../types"

export class ParserStack {
  /** The actual array stack. */
  declare stack: ParserElementStack

  /** @param stack - The stack to use as the starting state, which will be cloned. */
  constructor(stack: ParserElementStack = []) {
    this.stack = klona(stack)
  }

  /** The size of the stack. */
  get length() {
    return this.stack.length
  }

  /** Add a child to every element. */
  increment() {
    this.stack.forEach(element => element[2]++)
  }

  /**
   * Add a new element.
   *
   * @param id - The node type of the token.
   * @param start - The start position of the token.
   * @param children - The number of children the token will start with.
   */
  push(id: number, start: number, children: number) {
    this.stack.push([id, start, children])
  }

  /** Remove and return the last element. */
  pop() {
    return this.stack.pop()
  }

  /** Remove every element past the index given. */
  close(idx: number) {
    this.stack = this.stack.slice(0, idx + 1)
  }

  /** Returns the last element with the given ID. */
  last(id: number) {
    const idx = this.stack.map(element => element[0]).lastIndexOf(id)
    if (idx === -1) return null
    return idx
  }

  /** Returns a safe copy of the stack's internal array. */
  serialize() {
    return klona(this.stack)
  }
}
