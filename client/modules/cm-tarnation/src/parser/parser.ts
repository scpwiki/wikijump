import type { EditorParseContext } from "@codemirror/language"
import { Input, Tree } from "lezer-tree"
import type { TarnationLanguage } from "../language"
import type { EmbedToken, MappedToken, Token } from "../types"
import { ParserContext } from "./context"
import { EmbeddedHandler } from "./embedded-handler"

export class Parser {
  /** Handler instance for parsing embedded languages. */
  private declare embeddedHandler: EmbeddedHandler

  /** Current parser context. Gets mutated as the parser advances. */
  declare context: ParserContext

  /** Tokens to parse. */
  declare pending: Token[]

  /**
   * @param language - The host language.
   * @param context - The context/state to use.
   * @param input - The document that was tokenized.
   * @param pending - The tokens to parse.
   * @param editorContext - The CodeMirror editor parse context to use.
   */
  constructor(
    language: TarnationLanguage,
    context: ParserContext,
    input: Input,
    pending: Token[] = [],
    editorContext?: EditorParseContext
  ) {
    this.context = context
    this.pending = pending
    this.embeddedHandler = new EmbeddedHandler(language, context, input, editorContext)
  }

  /** True if the parser has finished parsing already. */
  get done() {
    return this.context.index >= this.pending.length && this.embeddedHandler.done
  }

  /** Executes a parse step. */
  private parse() {
    const token = this.pending[this.context.index]

    if (!token) return

    switch (typeof token[0]) {
      // embed token
      case "string": {
        this.embeddedHandler.push(token as EmbedToken)
        break
      }
      // mapped token
      case "number": {
        const ctx = this.context
        const [type, from, to, open, close] = token as MappedToken

        // token representing an embedded language location
        if (type === -1) {
          const index = ctx.buffer.add([0, from, to, -1, Tree.empty])
          ctx.stack.increment()
          this.embeddedHandler.push(index)
          break
        }

        // add open nodes to stack
        // this doesn't affect the buffer at all, but now we can watch for
        // when another node closes one of the open nodes we added
        if (open) {
          for (let i = 0; i < open.length; i++) {
            const [id, inclusive] = open[i]
            ctx.stack.push(id, inclusive ? from : to, type ? (inclusive ? 0 : -1) : 0)
          }
        }

        // we don't want to push the actual token twice
        let pushed = false

        // pop close nodes from the stack, if they can be paired with an open node
        if (close) {
          for (let i = 0; i < close.length; i++) {
            const [id, inclusive] = close[i]
            const idx = ctx.stack.last(id)

            if (idx !== null) {
              // cut off anything past the closing element
              // i.e. inside nodes won't persist outside their parent if they
              // never closed before their parent did
              ctx.stack.close(idx)

              // if we're inclusive of the closing token we need to push the token early
              if (type && inclusive && !pushed) {
                ctx.buffer.add([type, from, to, 4])
                ctx.stack.increment()
                pushed = true
              }

              // finally pop the node
              const [node, pos, children] = ctx.stack.pop()!
              ctx.buffer.add([node, pos, inclusive ? to : from, children * 4 + 4])
              ctx.stack.increment()
            }
          }
        }

        // push the actual token to the buffer, if it hasn't been already
        if (type && !pushed) {
          ctx.buffer.add([type, from, to, 4])
          ctx.stack.increment()
        }

        break
      }
    }

    this.context.index += 1
  }

  /**
   * Advances the parser. Returns null if it isn't done, otherwise returns
   * a buffer and reused tree nodes.
   */
  advance() {
    if (this.context.index < this.pending.length) this.parse()
    if (!this.embeddedHandler.done) this.embeddedHandler.advance()

    // check if complete
    if (this.context.index >= this.pending.length && this.embeddedHandler.done) {
      return this.context.buffer.compile()
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

  /**
   * Forces the parser to advance fully and return a tree, but avoids
   * advancing embedded languages, instead calling their parser's
   * `forceFinish` method.
   */
  forceFinish() {
    while (this.context.index < this.pending.length) this.parse()
    while (!this.embeddedHandler.done) this.embeddedHandler.advance(true)
    return this.context.buffer.compile()
  }

  /**
   * Forces a full parse on a list of tokens. This purposefully avoids
   * affecting the parser's state. Requires that the parser not be in the
   * middle of parsing another set of tokens.
   */
  forceTokens(pending: Token[]) {
    if (this.context.index !== 0) {
      throw new Error("Can't force while running another parse!")
    }
    const context = this.context.serialize()
    const oldPending = this.pending

    this.pending = pending
    const result = this.forceFinish()

    this.context = ParserContext.deserialize(context)
    this.embeddedHandler.set(this.context)
    this.pending = oldPending

    return result
  }
}
