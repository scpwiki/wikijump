import type { Input } from "@lezer/common"
import type { Grammar, GrammarToken } from "../grammar/grammar"
import type { TarnationLanguage } from "../language"
import type { NodeMap } from "../node-map"
import { ParseRegion } from "../region"
import type { MappedToken, Token } from "../types"
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

  /** Host node map, mapping string names to node ids. */
  private declare nodes: NodeMap

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
    if (!language.grammar || !language.nodes) {
      throw new Error("Unloaded language provided to tokenizer!")
    }

    this.context = context
    this.buffer = buffer
    this.grammar = language.grammar
    this.nodes = language.nodes
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
   * Compiles a `GrammarToken` into a `MappedToken`. Remaps type names to node IDs.
   *
   * @param t - The token to be converted.
   */
  private compileGrammarToken(t: GrammarToken): MappedToken {
    // this function gets ran for literally every token so I've elected to take
    // the most insane, most verbose and fastest approach I could come up with.
    // this mainly involves completely avoiding iterator methods,
    // such as destructuring, mapping functions, etc.

    // also, if you think i'm trying to just outsmart the compiler - I'm not.
    // I have benchmarked this. this really is faster, and I hate it

    const out: MappedToken = [this.nodes.get(t.type)!, t.from, t.to]

    if (t.open) {
      out[3] = []
      for (let i = 0; i < t.open.length; i++) {
        out[3][i] = [this.nodes.get(t.open[i][0])!, t.open[i][1]]
      }
    }

    if (t.close) {
      out[4] = []
      for (let i = 0; i < t.close.length; i++) {
        out[4][i] = [this.nodes.get(t.close[i][0])!, t.close[i][1]]
      }
    }

    return out
  }

  /**
   * Returns if the `last` `MappedToken` is effectively equivalent to the
   * `next` `GrammarToken`, as in the two can be merged without any loss of
   * information.
   *
   * @param last - The token to check if it can be potentially extended.
   * @param next - The new token which may be able to merge into the `last` token.
   */
  private canContinue(last?: MappedToken, next?: GrammarToken) {
    if (!last || !next) return false // tokens are invalid
    // parser directives present
    if (last.length > 2 || next.open || next.close) return false
    // embedded handling token
    if (last[0] === -1 || next.embedded) return false
    // types aren't equivalent
    if (last[0] !== this.nodes.get(next.type)) return false
    // tokens aren't inline
    if (last[2] !== next.from) return false
    // tokens are effectively equivalent
    return true
  }

  /** Gets the grammar's match at the current tokenizer position. */
  private match() {
    const ctx = this.context
    const match = this.grammar.match(
      { state: ctx.stack.state, context: ctx.stack.context },
      this.str,
      ctx.pos - this.offset,
      ctx.pos
    )
    if (!match) return { tokens: null, length: 1 } // always advance
    const tokens = match.compile()
    if (!tokens.length) return { tokens: null, length: match.length || 1 }
    return { tokens, length: match.length }
  }

  /** Executes a tokenization step. */
  private tokenize() {
    const stack = this.context.stack

    const { tokens, length } = this.match()

    this.context.pos += length

    if (!tokens) return null

    const mapped: Token[] = []

    let last!: MappedToken

    for (let idx = 0; idx < tokens.length; idx++) {
      const t = tokens[idx]

      let changedStack = t.next || t.switchTo || t.embedded || t.context

      let pushEmbedded = false

      if (t.embedded) {
        // token represents the entire region, not the start or end of one
        if (!stack.embedded && t.embedded.endsWith("!")) {
          const lang = t.embedded.slice(0, t.embedded.length - 1)
          mapped.push([lang, t.from, t.to])
          continue
        }
        // token ends an embedded region
        else if (t.embedded === "@pop") {
          const range = stack.endEmbedded(t.from)
          if (range) mapped.push(range)
        }
        // token starts an embedded region
        else if (!stack.embedded) {
          pushEmbedded = true
          stack.setEmbedded(t.embedded, t.to)
        }
      }

      // handle stack manipulation and match context changes
      if (t.next) {
        // prettier-ignore
        switch (t.next) {
          case "@pop":    stack.pop()                        ; break
          case "@popall": stack.popall()                     ; break
          case "@push":   stack.push(stack.state, t.context) ; break
          default:        stack.push(t.next, t.context)
        }
      } else if (t.switchTo) {
        stack.switchTo(t.switchTo, t.context)
      } else if (t.context) {
        stack.context = t.context
      }

      // check if the new token can be merged into the last one
      if (!t.empty && (!stack.embedded || pushEmbedded)) {
        if (last && !changedStack && this.canContinue(last, t)) last[2] = t.to
        else mapped.push((last = this.compileGrammarToken(t)))
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
      const stack = this.context.stack.serialize()
      const tokens = this.tokenize()
      if (tokens?.length) this.buffer.add(pos, stack, tokens)
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
        this.buffer.ensureLast(this.context.pos, this.context.stack)
        this.context = this.buffer.last!.context
        return true
      }
    }

    return false
  }
}
