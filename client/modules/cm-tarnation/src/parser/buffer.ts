import type { Tree } from "@lezer/common"
import { search } from "@wikijump/util"
import type { LezerToken } from "../types"

/**
 * A `ParserBuffer` stores a flat list of tokens for the Tarnation parser.
 * This flat list is steadily expanded as the parser advances, until
 * eventually it decides that it is done and calls the `compile` method.
 * This compiles the buffer's list of tokens into a format that can be
 * consumed by CodeMirror.
 */
export class ParserBuffer {
  /** The actual array of tokens that this buffer manages. */
  declare buffer: LezerToken[]

  /** @param tokens - Tokens to start the buffer with. */
  constructor(tokens: LezerToken[] = []) {
    this.buffer = tokens
  }

  /** Number of elements in the buffer. */
  get length() {
    return this.buffer.length
  }

  /** The last node in the buffer. */
  get last() {
    return this.buffer[this.buffer.length - 1]
  }

  /** Fully compiles the buffer's data into a `Tree.build` compatible format. */
  compile() {
    const buffer: number[] = []
    const reused: Tree[] = []

    for (let i = this.buffer.length - 1; i >= 0; i--) {
      const token = this.buffer[i]
      if (!token[4]) {
        buffer.push(token[3], token[2], token[1], token[0])
      } else {
        const [, start, end, size, tree] = token
        reused.push(tree)
        const idx = reused.length - 1
        buffer.push(-1, start + tree.length, start, idx)
        // skip past cached tree
        if (size >= 8) {
          let left = (size - 4) / 4
          while (left !== 0) {
            i--, left--
            // add a filler/repeat node using context hash
            buffer.push(-2, end, start, 0)
          }
        }
      }
    }

    // our data is inverted (push+reverse is faster than unshift usually)
    buffer.reverse()

    return { buffer, reused }
  }

  /**
   * Adds a token to the buffer, and returns its assigned index.
   *
   * @param token - The token to add.
   */
  add(token: LezerToken): number
  /**
   * Adds a token to the buffer, and returns its assigned index.
   *
   * @param id - The node ID of the token.
   * @param from - The start position of the token.
   * @param to - The end position of the token.
   * @param children - The number of nodes prior that are children of this token.
   * @param tree - An optional precompiled tree that will replace this node.
   */
  add(id: number, from: number, to: number, children: number): number
  add(
    id: number | LezerToken,
    from?: number,
    to?: number,
    children?: number,
    tree?: Tree
  ): number {
    if (typeof id === "number") {
      if (!from || !to || !children) throw new Error("Invalid token!")
      this.buffer.push([id, from, to, children, tree])
    } else {
      this.buffer.push(id)
    }
    return this.buffer.length - 1
  }

  /** Retrieves a token from the buffer. */
  get(idx: number): LezerToken | null {
    return this.buffer[idx] ?? null
  }

  /**
   * Determines if a token is in the buffer. Returns the token's index if
   * it was found.
   */
  has(token: LezerToken) {
    const result = this.buffer.indexOf(token)
    return result === -1 ? false : result
  }

  /** Binary search comparator function. */
  private searchCmp = ([, pos]: LezerToken, target: number) => {
    return pos === target || pos - target
  }

  /**
   * Searches for the closest token to the given position.
   *
   * @param pos - The position to find.
   * @param side - The side to search on. -1 is left (before), 1 is right
   *   (after). 0 is the default, and it means either side.
   * @param precise - If true, the search will require an exact hit. If the
   *   search misses, it will return `null` for both the token and index.
   */
  search(pos: number, side: 1 | 0 | -1 = 0, precise = false) {
    const result = search(this.buffer, pos, this.searchCmp, { precise })
    if (!result) return { token: null, index: null }

    let { index } = result
    let token = this.buffer[index]

    // direct hit or we don't care about sidedness
    if (token[1] === pos || side === 0) return { token, index }

    // correct for sidedness
    while (token && (side === 1 ? token[1] < pos : token[1] > pos)) {
      index = side === 1 ? index + 1 : index - 1
      token = this.buffer[index]
    }

    // no valid tokens
    if (!token) return { token: null, index: null }

    return { token, index }
  }

  /**
   * Clones the buffer. This clones every token - but does not clone any of
   * their trees. Trees are intended to be immutable, so this is safe to do.
   *
   * @param raw - If true, the returned clone will be the raw buffer rather
   *   than a new `ParserBuffer` instance.
   */
  clone(): ParserBuffer
  clone(raw: true): LezerToken[]
  clone(raw = false): ParserBuffer | LezerToken[] {
    const cloneBuffer: LezerToken[] = []
    for (let idx = 0; idx < this.buffer.length; idx++) {
      cloneBuffer[idx] = this.buffer[idx].slice(0) as LezerToken
    }
    return raw ? cloneBuffer : new ParserBuffer(cloneBuffer)
  }

  /**
   * Returns a shallow clone of the internal buffer.
   *
   * @param upto - If given, only tokens whose positions are less than this
   *   value will be serialized.
   */
  shallow(upto?: number) {
    if (upto === undefined) {
      // equivalent to:
      // return [...this.buffer]
      // but this is faster
      return this.buffer.slice(0)
    } else {
      const { index } = this.search(upto, 1)
      if (!index) return this.buffer.slice(0)
      return this.buffer.slice(0, index + 1)
    }
  }
}
