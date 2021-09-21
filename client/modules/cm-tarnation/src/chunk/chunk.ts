import { GrammarState } from "../grammar/state"
import * as Token from "../token"
import type { GrammarToken } from "../types"
import type { ParseStack } from "./parsing"

/**
 * A `Chunk` stores tokens emitted by the tokenization into discrete, well,
 * chunks. Chunks store their token positions (as in, position in the
 * document) relative to the chunk's own "position". This allows the chunk
 * to be moved anywhere in the document, and have the tokens follow. What
 * this means is that a chunk can be adjusted slightly forward or back as
 * the document changes, allowing for them to be reused when tokenizing.
 */
export class Chunk {
  /** The chunk's starting position. */
  private declare _pos: number

  /** The chunk's relative extent, as determined from the positions of its tokens. */
  private declare _max: number

  /** The tokens stored in this chunk. */
  declare tokens: ArrayBuffer[]

  /** State at the start of this chunk. */
  declare state: GrammarState

  /**
   * If this chunk has been parsed, this property will have the result of
   * that parse cached.
   */
  declare parsed: null | {
    /** Parsed tokens from this chunk. */
    tokens: ArrayBuffer
    /** Stack state at the end of this chunk. */
    stack: ParseStack
  }

  /**
   * @param pos - Position of this chunk.
   * @param state - The state for the start of this chunk.
   * @param tokens - The grammar tokens to store in this chunk.
   */
  constructor(pos: number, state: GrammarState, tokens: GrammarToken[] = []) {
    this._pos = pos
    this.state = state
    this._max = 0
    this.parsed = null
    this.setTokens(tokens)
  }

  /** The chunk's starting position. */
  get pos() {
    return this._pos
  }

  /** The chunk's starting position. */
  set pos(pos: number) {
    this.parsed = null
    this._pos = pos
  }

  /** The chunk's maximum extent, as determined from the positions of its tokens. */
  get max() {
    return this._max + this._pos
  }

  /**
   * Adds a token to the chunk.
   *
   * @param token - The token to add.
   */
  add(token: GrammarToken) {
    this.parsed = null

    // make token relative to chunk position
    const from = token[1] - this._pos
    const to = token[2] - this._pos

    if (to > this._max) this._max = to

    this.tokens.push(Token.create(token[0], from, to, token[3], token[4]))
  }

  /**
   * Sets the chunk's tokens.
   *
   * @param tokens - The tokens to add.
   */
  setTokens(tokens: GrammarToken[]) {
    this.parsed = null
    this.tokens = []
    this._max = 0
    for (let idx = 0; idx < tokens.length; idx++) {
      this.add(tokens[idx])
    }
  }

  /** Returns a deep clone of the chunk. */
  clone() {
    const chunk = new Chunk(this.pos, this.state.clone())
    chunk.tokens = this.tokens.slice()
    chunk._max = this._max
    return chunk
  }

  /**
   * Determines if a grammar's state (and parse position) is compatible
   * with reusing this node. This is only a safe determination if it is
   * made *after* the changed range of the document.
   *
   * @param state - The state to compare against.
   * @param pos - The position to compare against.
   * @param offset - The edit offset, to correct for chunk position differences.
   */
  isReusable(state: GrammarState, pos: number, offset = 0) {
    if (this._pos + offset !== pos) return false
    if (!state.equals(this.state)) return false
    return true
  }
}
