import { dequal } from "dequal"
import { klona } from "klona"
import type { SerializedTokenizerStack, Token } from "../types"
import { TokenizerContext } from "./context"
import { TokenizerStack } from "./stack"

export class Chunk {
  /** Serialized state of the stack at the start of this chunk. */
  private declare _stack: SerializedTokenizerStack

  /** The tokens stored in this chunk. */
  private declare _tokens: Token[]

  /** The chunk's starting position. */
  private declare _pos: number

  /**
   * A cached result of this chunk's last compile. Gets invalidated if the
   * position or token array is manipulated.
   */
  private declare compiled?: Token[]

  /** A weakmap cache for compiled tokens. */
  private cache = new WeakMap<Token, Token>()

  /**
   * @param pos - Position of this chunk.
   * @param stack - The state of the stack for the start of this chunk.
   * @param tokens - The mapped tokens to store in this chunk.
   * @param relativeTo - Indicates the position the tokens are relative to, if any.
   */
  constructor(
    pos: number,
    stack: TokenizerStack | SerializedTokenizerStack = { stack: [], embedded: null },
    tokens: Token[] = [],
    relativeTo?: number
  ) {
    this._pos = pos
    this.stack = stack
    this.setTokens(tokens, relativeTo)
  }

  /** The chunk's starting position. */
  get pos() {
    return this._pos
  }

  /** The chunk's starting position. */
  set pos(pos: number) {
    // reset cache if the position is changed
    this.cache = new WeakMap()
    this.compiled = undefined
    this._pos = pos
  }

  /** The chunk's start position stack (not serialized). */
  get stack() {
    return new TokenizerStack(this._stack)
  }

  /** The chunk's start position stack. */
  set stack(stack: TokenizerStack | SerializedTokenizerStack) {
    if ("serialize" in stack) stack = stack.serialize()
    this._stack = stack as SerializedTokenizerStack
  }

  /** Number of tokens stored in this chunk. */
  get size() {
    return this._tokens.length
  }

  /** The context for this chunk. */
  get context() {
    return new TokenizerContext(this._pos, new TokenizerStack(this._stack))
  }

  /**
   * Adds a token to the chunk.
   *
   * @param token - The token to add.
   * @param relativeTo - Indicates the position the token is already
   *   relative to, if any.
   */
  add(token: Token, relativeTo?: number) {
    this.compiled = undefined

    let [type, from, to, open, close] = token

    // undo the token being relative to some other position
    if (relativeTo) {
      from += relativeTo
      to += relativeTo
    }

    // make token relative to chunk position
    from -= this._pos
    to -= this._pos

    if (typeof type !== "string") {
      this._tokens.push([type, from, to, open, close])
    } else {
      this._tokens.push([type, from, to])
    }
  }

  /**
   * Sets the chunk's tokens.
   *
   * @param tokens - The tokens to add.
   * @param relativeTo - Indicates the position the tokens are already
   *   relative to, if any.
   */
  setTokens(tokens: Token[], relativeTo?: number) {
    this.compiled = undefined

    this._tokens = []
    for (let idx = 0; idx < tokens.length; idx++) {
      this.add(tokens[idx], relativeTo)
    }
  }

  /** Compiles a token, or returning it from the cache if available. */
  private compileToken(token: Token) {
    if (this.cache.has(token)) return this.cache.get(token)!

    const compiled: Token =
      typeof token[0] !== "string"
        ? [token[0], token[1] + this._pos, token[2] + this._pos, token[3], token[4]]
        : [token[0], token[1] + this._pos, token[2] + this._pos]

    this.cache.set(token, compiled)

    return compiled
  }

  /** Returns the chunk's stored tokens. */
  compile() {
    if (this.compiled) return this.compiled

    const tokens: Token[] = []
    for (let idx = 0; idx < this._tokens.length; idx++) {
      const token = this._tokens[idx]
      if (!token[0] && !token[3] && !token[4]) continue
      tokens.push(this.compileToken(token))
    }

    this.compiled = tokens
    return tokens
  }

  /** Returns a deep clone of the chunk. */
  clone() {
    return new Chunk(this.pos, klona(this._stack), klona(this._tokens), this.pos)
  }

  /**
   * Determines if the tokenizer's current state is compatible with reusing
   * this node. This is only a safe determination if it is made *after* the
   * changed range of the document.
   *
   * @param context - The context to compare against.
   */
  isReusable(context: TokenizerContext) {
    return this._pos === context.pos && dequal(this._stack, context.stack.serialize())
  }
}
