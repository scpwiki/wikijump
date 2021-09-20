import type { Tree } from "@lezer/common"
import type { Chunk } from "./chunk"
import type { LezerToken } from "./types"
import { cloneNestedArray, getEmbeddedParserNode } from "./util"

/** Stack used by the parser to track tree construction. */
export type ParseElementStack = [name: number, start: number, children: number][]

/**
 * A `ParseStack` keeps track of opened nodes destined to be eventually
 * closed. Any number of nodes can be open, and this is how parsing
 * actually creates a tree with depth.
 */
export class ParseStack {
  /** The actual array stack. */
  declare stack: ParseElementStack

  /** @param stack - The stack to use as the starting state, which will be cloned. */
  constructor(stack: ParseElementStack) {
    this.stack = cloneNestedArray(stack)
  }

  /** Add a child to every element. */
  increment() {
    for (let idx = 0; idx < this.stack.length; idx++) {
      this.stack[idx][2]++
    }
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
    let last = -1
    for (let idx = 0; idx < this.stack.length; idx++) {
      const elementID = this.stack[idx][0]
      if (elementID === id) last = idx
    }
    if (last === -1) return null
    return last
  }

  /** Returns a clone of this stack. */
  clone() {
    return new ParseStack(this.stack)
  }
}

/**
 * Parses a {@link Chunk}, returning the parsed out {@link LezerToken}s.
 * Mutates the given {@link ParseStack}, and caches the resultant tokens and
 * stack into the {@link Chunk} as well.
 *
 * @param stack - The stack to use for parsing.
 * @param chunk - The chunk to parse.
 */
export function parseChunk(stack: ParseStack, chunk: Chunk) {
  const buffer: LezerToken[] = []
  const tokens = chunk.compile()

  for (let idx = 0; idx < tokens.length; idx++) {
    const t = tokens[idx]
    switch (typeof t[0]) {
      // embed token
      case "string": {
        const node = getEmbeddedParserNode(t[0], t[1], t[2])
        const token: LezerToken = [0, t[1], t[2], -1, node]
        buffer.push(token)
        stack.increment()
        break
      }
      // grammar token (default)
      default: {
        /*
         * this upcoming code entirely avoids iterator methods, like destructuring
         * thus, it's entirely unreadable
         * I've left comments describing what a destructured approach looks like,
         * but without actually using it.
         * doing it this way has a decent speed boost but yeah it looks awful
         */

        // const [type, from, to, open, close] = token

        // add open nodes to stack
        // this doesn't affect the buffer at all, but now we can watch for
        // when another node closes one of the open nodes we added
        // if (open) {
        if (t[3]) {
          // for (let i = 0; i < open.length; i++) {
          for (let i = 0; i < t[3].length; i++) {
            // const [id, inclusive] = open[i]
            const o = t[3][i]
            // stack.push(
            //   id,
            //   inclusive ? from : to,
            //   type ? (inclusive ? 0 : -1) : 0
            // )
            // prettier-ignore
            stack.push(
              o[0],
              o[1] ? t[1] : t[2],
              t[0] ? (o[1] ? 0 : -1) : 0
            )
          }
        }

        // we don't want to push the actual token twice
        let pushed = false

        // pop close nodes from the stack, if they can be paired with an open node
        // if (close) {
        if (t[4]) {
          // for (let i = 0; i < close.length; i++) {
          for (let i = 0; i < t[4].length; i++) {
            // const [id, inclusive] = close[i]
            const c = t[4][i]
            // const idx = stack.last(id)
            const idx = stack.last(c[0])

            if (idx !== null) {
              // cut off anything past the closing element
              // i.e. inside nodes won't persist outside their parent if they
              // never closed before their parent did
              stack.close(idx)

              // if inclusive of the closing token we need to push the token early
              // if (type && inclusive && !pushed) {
              if (t[0] && c[1] && !pushed) {
                // buffer.push([type, from, to, 4])
                buffer.push([t[0], t[1], t[2], 4])
                stack.increment()
                pushed = true
              }

              // finally pop the node
              // const [node, pos, children] = ctx.stack.pop()!
              const s = stack.pop()!
              // buffer.push([
              //   node,
              //   pos,
              //   inclusive ? to : from, children * 4 + 4
              // ])
              // prettier-ignore
              buffer.push([
                s[0],
                s[1],
                c[1] ? t[2] : t[1], s[2] * 4 + 4
              ])
              stack.increment()
            }
          }
        }

        // push the actual token to the buffer, if it hasn't been already
        // if (type && !pushed) {
        if (t[0] && !pushed) {
          // buffer.push([type, from, to, 4])
          buffer.push([t[0], t[1], t[2], 4])
          stack.increment()
        }

        break
      }
    }
  }

  // cache result
  chunk.parsed = { tokens: buffer, stack: stack.clone() }

  return buffer
}

/**
 * Compiles, and if needed, parses, a list of {@link Chunk}s. Returns a
 * `Tree.build` compatible buffer and a list of "reused" trees for language nesting.
 *
 * @param chunks - The chunks to compile.
 * @param startStack - The stack to use for parsing, if given. Otherwise,
 *   an empty stack will be created.
 */
export function compileChunks(chunks: Chunk[], startStack?: ParseStack) {
  const buffer: number[] = []
  const reused: Tree[] = []

  let stack = startStack || new ParseStack([])
  let shouldCloneStack = false

  for (let i = 0; i < chunks.length; i++) {
    const chunk = chunks[i]

    let tokens: LezerToken[]
    if (chunk.parsed) {
      tokens = chunk.parsed.tokens
      stack = chunk.parsed.stack
      shouldCloneStack = true
    } else {
      if (shouldCloneStack) stack = stack.clone()
      tokens = parseChunk(stack, chunk)
      shouldCloneStack = false
    }

    for (let j = 0; j < tokens.length; j++) {
      const t = tokens[j]
      if (!t[4]) buffer.push(t[0], t[1], t[2], t[3])
      else {
        // push nest node tree
        reused.push(t[4])
        buffer.push(reused.length - 1, t[1], t[1] + t[4].length, -1)
      }
    }
  }

  return { buffer, reused }
}
