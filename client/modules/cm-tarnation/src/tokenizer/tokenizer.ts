import type { Input } from "lezer-tree"
import type { Grammar, GrammarToken } from "../grammar/grammar"
import type { TarnationLanguage } from "../language"
import type { NodeMap } from "../node-map"
import type { MappedToken, ParseRegion, SerializedTokenizerStack, Token } from "../types"
import type { TokenizerBuffer } from "./buffer"
import type { Chunk } from "./chunk"
import type { TokenizerContext } from "./context"
import type { TokenizerStack } from "./stack"

/** Amount of safety-margin to read after the tokenizer's end position. */
const MARGIN_AFTER = 500

export class Tokenizer {
  /** Host grammar. */
  private declare grammar: Grammar

  /** Host node map, mapping string names to node ids. */
  private declare nodes: NodeMap

  /** String that is actually being tokenized. */
  private declare str: string

  /** Starting offset, as in where in the editor document does the string start. */
  private declare offset: number

  /** Position where the tokenizer should stop after reaching. */
  private declare end: number

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
    this.end = region.to
  }

  /** True if the tokenizer has already completed. */
  get done() {
    return this.context.pos > this.end
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
    // prettier-ignore
    const {
      str, offset,
      context: { pos, stack: { state, context } }
    } = this

    const match = this.grammar.match({ state, context }, str, pos - offset, pos)
    if (!match) return { tokens: null, length: 1 } // always advance

    const tokens = match.compile()
    if (!tokens.length) return { tokens: null, length: match.length || 1 }

    return { tokens, length: match.length }
  }

  /** Executes a tokenization step. */
  private tokenize() {
    const { stack } = this.context

    const { tokens, length } = this.match()

    const startPos = this.context.pos
    let startStack: TokenizerStack | SerializedTokenizerStack = stack

    this.context.pos += length

    if (!tokens) return { tokens, startPos, startStack }

    let changedStack = false
    for (let idx = 0; idx < tokens.length; idx++) {
      const token = tokens[idx]
      if (token.next || token.switchTo || token.context || token.embedded) {
        changedStack = true
        startStack = stack.serialize()
        break
      }
    }

    const mapped = new Set<Token>()

    let last!: MappedToken

    for (let idx = 0; idx < tokens.length; idx++) {
      const token = tokens[idx]
      const { next, switchTo, embedded, context, from, to } = token

      let pushEmbedded = false

      if (embedded) {
        // token starts an embedded region
        if (!stack.embedded && embedded.endsWith("!")) {
          const lang = embedded.slice(0, embedded.length - 1)
          mapped.add((last = [-1, from, to]))
          mapped.add([lang, from, to])
          continue
        }
        // token ends an embedded region
        else if (embedded === "@pop") {
          const range = stack.endEmbedded(from)
          if (range) mapped.add(range)
        }
        // token represents the entire region, not the start or end of one
        else if (!stack.embedded) {
          pushEmbedded = true
          stack.setEmbedded(embedded, to)
        }
      }

      // handle stack manipulation and match context changes
      if (next) {
        // prettier-ignore
        switch (next) {
          case "@pop":    stack.pop()                      ; break
          case "@popall": stack.popall()                   ; break
          case "@push":   stack.push(stack.state, context) ; break
          default:        stack.push(next, context)
        }
      } else if (switchTo) {
        stack.switchTo(switchTo, context)
      } else if (context) {
        stack.context = context
      }

      // check if the new token can be merged into the last one
      if (!token.empty && (!stack.embedded || pushEmbedded)) {
        if (last && !changedStack && this.canContinue(last, token)) last[2] = token.to
        else mapped.add((last = this.compileGrammarToken(token)))
      }

      // add a token for marking the embedded language
      if (pushEmbedded) mapped.add((last = [-1, to, to]))
    }

    return { tokens: mapped, startPos, startStack }
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
    const { context: ctx, end, buffer } = this

    if (ctx.pos < end) {
      const { tokens, startPos, startStack } = this.tokenize()
      if (tokens) buffer.add(startPos, startStack, ...tokens)
    }

    if (ctx.pos >= end) return this.chunks

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
        this.buffer.link(right)
        this.context = this.buffer.last!.context
        return true
      }
    }

    return false
  }
}
