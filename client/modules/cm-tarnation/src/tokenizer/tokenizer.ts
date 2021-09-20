import { Nesting } from "../enums"
import type { Grammar } from "../grammar/grammar"
import type { TarnationLanguage } from "../language"
import type { ParseRegion } from "../region"
import type { GrammarToken, Token } from "../types"
import { canContinue } from "../util"
import type { TokenizerBuffer } from "./buffer"
import type { TokenizerContext } from "./context"

const MARGIN_BEFORE = 32
const MARGIN_AFTER = 128

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
    region: ParseRegion
  ) {
    if (!language.grammar) throw new Error("Unloaded language provided to tokenizer!")

    this.context = context
    this.buffer = buffer
    this.grammar = language.grammar
    this.region = region
  }

  /** True if the tokenizer has already completed. */
  get done() {
    return this.context.pos >= this.region.to
  }

  /** The tokenizer's current chunks. */
  get chunks() {
    return this.buffer.buffer
  }

  /** Compiles the tokenizer's buffer. */
  compile() {
    return this.buffer.compile()
  }

  /** Advances the tokenizer. Returns null if it isn't done, otherwise returns true. */
  tokenize() {
    if (this.context.pos < this.region.to) {
      const pos = this.context.pos
      const startContext = this.context.clone()

      // tokenize

      const ctx = this.context

      let matchTokens: GrammarToken[] | null = null
      let length = 1

      const start = Math.max(pos - MARGIN_BEFORE, this.region.from)
      const startCompensated = this.region.compensate(pos, start - pos)

      const str = this.region.read(startCompensated, MARGIN_AFTER, this.region.to)

      const match = this.grammar.match(ctx.state, str, pos - start, pos)

      if (match) {
        ctx.state = match.state
        matchTokens = match.compile()
        length = match.length || 1
      }

      ctx.pos = this.region.compensate(pos, length)

      const tokens: Token[] = []

      if (matchTokens?.length) {
        let last!: GrammarToken

        for (let idx = 0; idx < matchTokens.length; idx++) {
          const t = matchTokens[idx]

          let pushNested = false

          if (t[5] !== undefined) {
            // token ends a nested region
            if (t[5] === Nesting.POP) {
              const range = ctx.endNested(t[1])
              if (range) tokens.push(range)
            }
            // token represents the entire region, not the start or end of one
            else if (!ctx.nested && t[5].endsWith("!")) {
              const lang = t[5].slice(0, t[5].length - 1)
              tokens.push([lang, t[1], t[2]])
              continue
            }
            // token starts a nested region
            else if (!ctx.nested) {
              pushNested = true
              ctx.startNested(t[5], t[2])
            }
          }

          if (!this.region.contiguous) {
            const from = this.region.compensate(pos, t[1] - pos)
            const end = this.region.compensate(pos, t[2] - pos)
            t[1] = from
            t[2] = end
          }

          // check if the new token can be merged into the last one
          if (!ctx.nested || pushNested) {
            if (last && canContinue(last, t)) last[2] = t[2]
            else tokens.push((last = t))
          }
        }
      }

      // add found tokens to buffer
      if (tokens?.length) this.buffer.add(pos, startContext, tokens)
    }

    if (this.context.pos >= this.region.to) return true

    return null
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
        this.buffer.link(right, this.region.original.length)
        this.buffer.ensureLast(this.context.pos, this.context)
        this.context = this.buffer.last!.context.clone()
        return true
      }
    }

    return false
  }
}
