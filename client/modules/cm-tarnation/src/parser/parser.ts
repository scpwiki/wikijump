import type { Tree } from "@lezer/common"
import { getEmbeddedParserNode } from ".."
import type { TarnationLanguage } from "../language"
import { ParseRegion } from "../region"
import type { Chunk } from "../tokenizer"
import type { LezerToken, MappedToken } from "../types"
import type { ParserContext } from "./context"

// not working quite right yet
const FINISH_INCOMPLETE_STACKS = false

/**
 * Tarnation's parser, which accepts tokenizer `Chunk` objects and parses
 * their tokens into a flat representation of a tree.
 *
 * It is designed to be very cheap and fast to use, which is because the
 * parser can only reuse data from behind the current parse position, and
 * not reuse ahead data. That means for the sake of performance that the
 * parser has to be very fast as it may need to recreate most of its parse
 * every time the document is changed.
 */
export class Parser {
  /** The host language. */
  private declare language: TarnationLanguage

  /** The region of the document that should be parsed. */
  private declare region: ParseRegion

  /** Current parser context. Gets mutated as the parser advances. */
  declare context: ParserContext

  /** Chunks to parse. */
  declare pending: Chunk[]

  /**
   * @param language - The host language.
   * @param context - The context/state to use.
   * @param region - The region of the document that should be parsed.
   * @param pending - The chunks to parse.
   */
  constructor(
    language: TarnationLanguage,
    context: ParserContext,
    region: ParseRegion,
    pending: Chunk[] = []
  ) {
    this.language = language
    this.context = context
    this.region = region
    this.pending = pending
  }

  get pos() {
    return this.pending?.[this.context.index - 1]?.max ?? this.region.from
  }

  get done() {
    return this.context.index >= this.pending.length || this.pos >= this.region.to
    // return this.context.index >= this.pending.length
  }

  /** Executes a parse step. */
  private parse() {
    const ctx = this.context
    const chunk = this.pending[ctx.index]

    // we want to cache the context before we process the chunk
    // this is the _starting_ context, not the ending context
    chunk.parserContext = this.context

    const tokens = chunk.compile()
    for (let idx = 0; idx < tokens.length; idx++) {
      // bit of a hack, but for some reason TS isn't typw narrowing the switch below
      // so we'll just act like this is a MappedToken, and then assert as any for
      // EmbedToken
      const t = tokens[idx] as MappedToken

      switch (typeof t[0]) {
        // embed token
        case "string": {
          const node = getEmbeddedParserNode(t[0], t[1], t[2])
          const token: LezerToken = [0, t[1], t[2], -1, node]
          ctx.buffer.add(token)
          ctx.stack.increment()
          break
        }
        // mapped token (default)
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
              // ctx.stack.push(
              //   id,
              //   inclusive ? from : to,
              //   type ? (inclusive ? 0 : -1) : 0
              // )
              // prettier-ignore
              ctx.stack.push(
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
              // const idx = ctx.stack.last(id)
              const idx = ctx.stack.last(c[0])

              if (idx !== null) {
                // cut off anything past the closing element
                // i.e. inside nodes won't persist outside their parent if they
                // never closed before their parent did
                ctx.stack.close(idx)

                // if inclusive of the closing token we need to push the token early
                // if (type && inclusive && !pushed) {
                if (t[0] && c[1] && !pushed) {
                  // ctx.buffer.add([type, from, to, 4])
                  ctx.buffer.add([t[0], t[1], t[2], 4])
                  ctx.stack.increment()
                  pushed = true
                }

                // finally pop the node
                // const [node, pos, children] = ctx.stack.pop()!
                const s = ctx.stack.pop()!
                // ctx.buffer.add([
                //   node,
                //   pos,
                //   inclusive ? to : from, children * 4 + 4
                // ])
                // prettier-ignore
                ctx.buffer.add([
                  s[0],
                  s[1],
                  c[1] ? t[2] : t[1], s[2] * 4 + 4
                ])
                ctx.stack.increment()
              }
            }
          }

          // push the actual token to the buffer, if it hasn't been already
          // if (type && !pushed) {
          if (t[0] && !pushed) {
            // ctx.buffer.add([type, from, to, 4])
            ctx.buffer.add([t[0], t[1], t[2], 4])
            ctx.stack.increment()
          }

          break
        }
      }
    }

    this.context.index += 1
  }

  /**
   * Takes the current parser context, clones it, and then makes sure that
   * every element on the stack has been resolved. Returns the cloned
   * context. If the stack is already complete, the current context will be
   * returned instead.
   */
  private finishIncompleteStack() {
    if (!FINISH_INCOMPLETE_STACKS) return this.context
    if (!this.context.stack.length) return this.context
    // temporary clone, we don't want to affect the existing state
    const ctx = this.context.clone()
    while (ctx.stack.length) {
      const [startID, startPos, children] = ctx.stack.pop()!
      ctx.buffer.add([startID, startPos, this.pos, children * 4 + 4])
      ctx.stack.increment()
    }
    return ctx
  }

  /**
   * Fully advances the parser, using the tokenizer's emitted chunks
   * directly with no additional parsing. This is for debug purposes, as it
   * allows distinguishing between bugs that orginate from either the
   * tokenizer or the parser.
   */
  advanceFullyRaw(): { buffer: number[]; reused: Tree[] } {
    const buffer: number[] = []
    for (let idx = 0; idx < this.pending.length; idx++) {
      const tokens = this.pending[idx].compile()
      for (let idx = 0; idx < tokens.length; idx++) {
        const t = tokens[idx]
        if (!t[0] || typeof t[0] === "string") continue
        buffer.push(t[0], t[1], t[2], 4)
      }
    }
    return { buffer, reused: [] }
  }

  /**
   * Advances the parser. Returns null if it isn't done, otherwise returns
   * a buffer and reused tree nodes.
   */
  advance() {
    if (!this.done) this.parse()
    // if (!this.embeddedHandler.done) this.embeddedHandler.advance()

    if (this.done) {
      const context = this.finishIncompleteStack()
      return context.buffer.compile()
    }

    return null
  }

  /**
   * Forces the parser to advance fully, which is rather expensive, and
   * returns the resultant buffer and reused nodes.
   */
  advanceFully() {
    let result: { buffer: number[]; reused: Tree[] } | null = null
    while ((result = this.advance()) === null) {}
    return result
  }
}
