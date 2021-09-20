import { klona } from "klona"
import { ParserContext } from "../parser"
import type { SerializedParserContext, Token } from "../types"
import type { TokenizerContext } from "./context"

/**
 * A `Chunk` stores tokens emitted by the tokenizer into discrete, well,
 * chunks. Chunks store their token positions (as in, position in the
 * document) relative to the chunk's own "position". This allows the chunk
 * to be moved anywhere in the document, and have the tokens follow. What
 * this means is that a chunk can be adjusted slightly forward or back as
 * the document changes, allowing for them to be reused when tokenizing.
 */
export class Chunk {
  /** Context at the start of this chunk. */
  private declare _context: TokenizerContext

  /** The tokens stored in this chunk. */
  private declare _tokens: Token[]

  /** The chunk's starting position. */
  private declare _pos: number

  /**
   * A cached result of this chunk's last compile. Gets invalidated if the
   * position or token array is manipulated.
   */
  private declare compiled?: Token[]

  /** The chunk's relative extent, as determined from the positions of its tokens. */
  private declare _max: number

  /**
   * A cached {@link ParserContext} for use by the parser. Used for reusing
   * left-hand parse data.
   */
  private declare _parserContext?: SerializedParserContext

  /**
   * @param pos - Position of this chunk.
   * @param context - The context for the start of this chunk.
   * @param tokens - The mapped tokens to store in this chunk.
   * @param relativeTo - Indicates the position the tokens are relative to, if any.
   */
  constructor(
    pos: number,
    context: TokenizerContext,
    tokens: Token[] = [],
    relativeTo?: number,
    parserContext?: ParserContext | SerializedParserContext
  ) {
    this._pos = pos
    this.context = context
    this._max = 0
    this.setTokens(tokens, relativeTo)
    if (parserContext) this.parserContext = parserContext
  }

  /** The chunk's starting position. */
  get pos() {
    return this._pos
  }

  /** The chunk's starting position. */
  set pos(pos: number) {
    this.compiled = undefined
    this._parserContext = undefined
    this._pos = pos
  }

  /** The chunk's maximum extent, as determined from the positions of its tokens. */
  get max() {
    return this._max + this._pos
  }

  /** Number of tokens stored in this chunk. */
  get size() {
    return this._tokens.length
  }

  /** The context for this chunk. */
  get context() {
    return this._context.clone()
  }

  set context(context: TokenizerContext) {
    this._context = context.clone()
  }

  /**
   * A cached {@link ParserContext} for use by the parser. Used for reusing
   * left-hand parse data.
   */
  get parserContext(): ParserContext | undefined {
    return this._parserContext
      ? ParserContext.deserialize(this._parserContext)
      : undefined
  }

  /**
   * A cached {@link ParserContext} for use by the parser. Used for reusing
   * left-hand parse data.
   */
  set parserContext(context: ParserContext | SerializedParserContext | undefined) {
    if (context === undefined) {
      this._parserContext = undefined
    } else {
      this._parserContext =
        context instanceof ParserContext ? context.serialize(this.max) : context
    }
  }

  /** True if this chunk has a {@link ParserContext} attached to it. */
  get hasParserContext() {
    return this._parserContext !== undefined
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
    this._parserContext = undefined

    let [type, from, to, open, close] = token

    // undo the token being relative to some other position
    if (relativeTo) {
      from += relativeTo
      to += relativeTo
    }

    // make token relative to chunk position
    from -= this._pos
    to -= this._pos

    if (to > this._max) this._max = to

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
    this._parserContext = undefined
    this._tokens = []
    this._max = 0
    for (let idx = 0; idx < tokens.length; idx++) {
      this.add(tokens[idx], relativeTo)
    }
  }

  /** Compiles a token. */
  private compileToken(token: Token): Token {
    return typeof token[0] !== "string"
      ? [token[0], token[1] + this._pos, token[2] + this._pos, token[3], token[4]]
      : [token[0], token[1] + this._pos, token[2] + this._pos]
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
    return new Chunk(
      this.pos,
      this.context,
      klona(this._tokens),
      this.pos,
      this.parserContext?.clone()
    )
  }

  /**
   * Determines if a tokenizer's state is compatible with reusing this
   * node. This is only a safe determination if it is made *after* the
   * changed range of the document.
   *
   * @param context - The context to compare against.
   * @param offset - The edit offset, to correct for chunk position differences.
   */
  isReusable(context: TokenizerContext, offset = 0) {
    if (this._pos + offset !== context.pos) return false
    if (!context.equals(this._context, offset)) return false
    return true
  }
}
