import type { Input } from "lezer-tree"
import type { Grammar, GrammarToken } from "../grammar/grammar"
import type { TarnationLanguage } from "../language"
import type { NodeMap } from "../node-map"
import type { EditRegion, MappedToken, Token } from "../types"
import type { TokenizerBuffer } from "./buffer"
import type { TokenizerContext } from "./context"

/** Amount of safety-margin to read before the tokenizer's start position. */
const MARGIN_BEFORE = 50
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
    region: EditRegion
  ) {
    if (!language.grammar || !language.nodes) {
      throw new Error("Unloaded language provided to tokenizer!")
    }

    this.context = context
    this.buffer = buffer
    this.grammar = language.grammar
    this.nodes = language.nodes

    const end = Math.min(region.to + MARGIN_AFTER, input.length)
    this.str = input.read(context.pos, end)
    this.offset = context.pos
    this.end = region.to
  }

  /** True if the tokenizer has already completed. */
  get done() {
    return this.context.pos > this.end
  }

  /**
   * Compiles a `GrammarToken` into a `MappedToken`. Remaps type names to node ids.
   *
   * @param token - The token to be converted.
   */
  private compileGrammarToken(token: GrammarToken): MappedToken {
    const nodes = this.nodes // makes the maps below easier to read
    const { type, from, to, open, close } = token

    const out: MappedToken = [nodes.get(type)!, from, to]

    if (open) out[3] = open.map(([type, inclusive]) => [nodes.get(type)!, inclusive])
    if (close) out[4] = close.map(([type, inclusive]) => [nodes.get(type)!, inclusive])

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

    this.context.pos += length

    if (!tokens) return null

    const mapped = new Set<Token>()

    let last!: MappedToken

    for (let idx = 0; idx < tokens.length; idx++) {
      const token = tokens[idx]
      const { next, switchTo, embedded, context, from, to } = token

      stack.changed = false
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
        if (last && !stack.changed && this.canContinue(last, token)) last[2] = token.to
        else mapped.add((last = this.compileGrammarToken(token)))
      }

      // add a token for marking the embedded language
      if (pushEmbedded) mapped.add((last = [-1, to, to]))
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
    const { context: ctx, end, buffer } = this

    if (ctx.pos < end) {
      const startContext = ctx.serialize()
      const tokens = this.tokenize()
      if (tokens) buffer.add(startContext, ...tokens)
    }

    if (ctx.pos >= end) return this.compile()

    return null
  }

  /**
   * Forces the tokenizer to advance fully, which is rather expensive, and
   * returns the resultant tokens.
   */
  advanceFully() {
    let result: Token[] | null = null
    while ((result = this.advance()) === null) {}
    return result
  }
}
