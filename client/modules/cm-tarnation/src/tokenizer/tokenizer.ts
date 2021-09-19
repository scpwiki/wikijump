import type { Input } from "@lezer/common"
import type { Grammar } from "../grammar/grammar"
import { GrammarToken, Nesting } from "../grammar/types"
import type { TarnationLanguage } from "../language"
import type { ParseRegion } from "../region"
import type { Token } from "../types"
import type { TokenizerBuffer } from "./buffer"
import type { Chunk } from "./chunk"
import type { TokenizerContext } from "./context"

/** Amount of safety-margin to read after the tokenizer's end position. */
const MARGIN_AFTER = 500

/**
 * The `Tokenizer` takes an input document and tokenizes it using a
 * grammar. During this process, it will convert all the grammar's emitted
 * tokens into more efficient and easily stored objects, along with other
 * housekeeping duties.
 *
 * Once it is done tokenizing, it will have populated a `TokenizerBuffer`
 * with a number of `Chunk` objects containing the finalized tokens. This
 * can then be pipelined into a `Parser`.
 */
export class Tokenizer {
  /** Host grammar. */
  private declare grammar: Grammar

  /** String that is actually being tokenized. */
  private declare str: string

  /** Starting offset, as in where in the editor document does the string start. */
  private declare offset: number

  /** The region of the document that should be tokenized. */
  private declare region: ParseRegion

  /** Tokenizer context/state. */
  declare context: TokenizerContext

  /** Tokenizer's token buffer, where matched tokens are cached. */
  declare buffer: TokenizerBuffer

  /**
   * @param language - Host language.
   * @param context - Tokenizer context/state.
   * @param buffer - Tokenizer buffer, either new or from a cache.
   * @param input - The document to tokenize.
   * @param region - The region of the document that should be tokenized.
   */
  constructor(
    language: TarnationLanguage,
    context: TokenizerContext,
    buffer: TokenizerBuffer,
    input: Input,
    region: ParseRegion
  ) {
    if (!language.grammar) throw new Error("Unloaded language provided to tokenizer!")

    this.context = context
    this.buffer = buffer
    this.grammar = language.grammar
    this.region = region

    const end = Math.min(region.to + MARGIN_AFTER, input.length)
    this.str = input.read(context.pos, end)
    this.offset = context.pos
  }

  /** True if the tokenizer has already completed. */
  get done() {
    return this.context.pos >= this.region.to
  }

  /** The tokenizer's current chunks. */
  get chunks() {
    return this.buffer.buffer
  }

  /**
   * Returns if the `last` `MappedToken` is effectively equivalent to the
   * `next` `GrammarToken`, as in the two can be merged without any loss of
   * information.
   *
   * @param last - The token to check if it can be potentially extended.
   * @param next - The new token which may be able to merge into the `last` token.
   */
  private canContinue(last?: GrammarToken, next?: GrammarToken) {
    if (!last || !next) return false // tokens are invalid
    // parser directives present
    if (last.length > 2 || next[3] || next[4]) return false
    // embedded handling token
    if (last[5] || next[5]) return false
    // types aren't equivalent
    if (last[0] !== next[0]) return false
    // tokens aren't inline
    if (last[2] !== next[1]) return false
    // tokens are effectively equivalent
    return true
  }

  /** Executes a tokenization step. */
  private tokenize() {
    let tokens: GrammarToken[] | null = null
    let length = 1

    const match = this.grammar.match(
      this.context.state,
      this.str,
      this.context.pos - this.offset,
      this.context.pos
    )

    if (match) {
      this.context.state = match.state
      tokens = match.compile()
      length = match.length || 1
    }

    this.context.pos += length

    if (!tokens?.length) return null

    const mapped: Token[] = []

    let last!: GrammarToken

    for (let idx = 0; idx < tokens.length; idx++) {
      const t = tokens[idx]

      let pushEmbedded = false

      if (t[5] !== undefined) {
        // token ends an embedded region
        if (t[5] === Nesting.POP) {
          const range = this.context.endEmbedded(t[1])
          if (range) mapped.push(range)
        }
        // token represents the entire region, not the start or end of one
        else if (!this.context.embedded && t[5].endsWith("!")) {
          const lang = t[5].slice(0, t[5].length - 1)
          mapped.push([lang, t[1], t[2]])
          continue
        }
        // token starts an embedded region
        else if (!this.context.embedded) {
          pushEmbedded = true
          this.context.setEmbedded(t[5], t[2])
        }
      }

      // check if the new token can be merged into the last one
      if (!this.context.embedded || pushEmbedded) {
        if (last && this.canContinue(last, t)) last[2] = t[2]
        else mapped.push((last = t))
      }
    }

    return mapped
  }

  /** Compiles the tokenizer's buffer. */
  compile() {
    return this.buffer.compile()
  }

  /**
   * Advances the tokenizer. Returns null if it isn't done, otherwise
   * returns a list of tokens.
   */
  advance() {
    if (this.context.pos < this.region.to) {
      const pos = this.context.pos
      const context = this.context.clone()
      const tokens = this.tokenize()
      if (tokens?.length) this.buffer.add(pos, context, tokens)
    }

    if (this.context.pos >= this.region.to) return this.chunks

    return null
  }

  /**
   * Forces the tokenizer to advance fully, which is rather expensive, and
   * returns the resultant tokens.
   */
  advanceFully() {
    let result: Chunk[] | null = null
    while ((result = this.advance()) === null) {}
    return result
  }

  /**
   * Tries to reuse a buffer *ahead* of the current position. Returns true
   * if this was successful, otherwise false.
   *
   * @param right - The buffer to try and reuse.
   */
  tryToReuse(right: TokenizerBuffer) {
    // can't reuse if we don't know the safe regions
    if (!this.region.edit) return false
    // can only safely reuse if we're ahead of the edited region
    if (this.context.pos <= this.region.edit.to) return false

    // check every chunk and see if we can reuse it
    for (let idx = 0; idx < right.buffer.length; idx++) {
      const chunk = right.buffer[idx]
      if (chunk.isReusable(this.context, this.region.edit.offset)) {
        right.slide(idx, this.region.edit.offset, true)
        this.buffer.link(right, this.region.length)
        this.buffer.ensureLast(this.context.pos, this.context)
        this.context = this.buffer.last!.context
        return true
      }
    }

    return false
  }
}
