import type { EditorParseContext } from "@codemirror/language"
import { Input, PartialParse, Tree } from "lezer-tree"
import type { TarnationLanguage } from "../language"
import type { EmbeddedData, EmbedToken, LezerToken } from "../types"
import type { ParserContext } from "./context"
import { EmbeddedLanguage } from "./embedded-language"

export class EmbeddedHandler {
  /** Token indexes pending a range to be allocated to them. */
  private pending: number[] = []

  /** Currently active parsers. */
  private parsers: {
    /** Index of the token representing the embedded region. */
    index: number
    /** The embedded language parser for the region. */
    lang: EmbeddedLanguage
    /** The parser for the region, if there is one. */
    parser?: PartialParse
  }[] = []

  /**
   * @param language - The host language.
   * @param context - The host parser context.
   * @param input - The document to give to parsers.
   * @param editorContext - The CodeMirror editor context - used to obtain
   *   the skipping parser, but is technically optional.
   */
  constructor(
    private language: TarnationLanguage,
    private context: ParserContext,
    private input: Input,
    private editorContext?: EditorParseContext
  ) {
    this.set(context)
  }

  /** Whether or not the handler has fully completed parsing. */
  get done() {
    return this.parsers.length === 0
  }

  /** Reset's the internal state to match the given context. */
  set(context: ParserContext) {
    this.context = context
    const embedded = context.embedded
    this.pending = [...embedded.pending]
    this.parsers = embedded.parsers.map(([index, range]) => ({
      lang: new EmbeddedLanguage(this.language, range),
      index
    }))
  }

  /**
   * Pushes a token or embedded range to allocate to a token.
   *
   * @param embed - A token index, token itself, or an embedded range token.
   */
  push(embed: number | LezerToken | EmbedToken) {
    // push token index
    if (typeof embed === "number") {
      this.pending.push(embed)
    }
    // push token
    else if (typeof embed[0] === "number") {
      const index = this.context.buffer.has(embed as LezerToken)
      if (!index) throw new Error("Invalid token index!")
      this.pending.push(index)
    }
    // push embedtoken
    else {
      if (this.pending.length === 0) {
        console.warn("Attempted to push an unassigned language! Ignoring...")
        return
      }
      const index = this.pending.shift()!
      const lang = new EmbeddedLanguage(this.language, embed as EmbedToken)
      this.parsers.push({ index, lang })
    }
  }

  /**
   * Advances a parser. Returns whether or not the parser given has finished.
   *
   * @param parser - The parser to advance.
   * @param force - Whether or not the parser should be forced to return a tree.
   */
  private advanceParser(
    parser: { index: number; lang: EmbeddedLanguage; parser?: PartialParse },
    force = false
  ) {
    const { index, lang } = parser
    const [, start, end] = lang.range
    const token = this.context.buffer.get(index)!

    if (!force && !lang.ready) return false

    if (token[4] !== Tree.empty && this.context.start >= end) {
      return true
    }

    try {
      let tree: Tree | null = null
      let parse: PartialParse

      if (!parser.parser) {
        parse = lang.startParse(this.input.clip(end), start, this.editorContext)
        parser.parser = parse
      } else {
        parse = parser.parser
      }

      if (!tree) {
        // effectively marks the tree as stale
        if (token[4] !== Tree.empty) token[4] = Tree.empty

        if (force) {
          const viewport = this.editorContext?.viewport
          // check if the region is visible, if it is, we're going to advance it fully
          // if not, we'll just try a forceFinish
          if (viewport && end > viewport.from && start < viewport.to) {
            while ((tree = parse.advance()) === null) {}
          } else {
            tree = parse.forceFinish()
          }
        } else {
          tree = parse.advance()
        }
      }

      if (tree) {
        token[0] = -1
        token[4] = tree
        return true
      }

      return false
    } catch (err) {
      console.warn(err)
      console.warn("Embedded language had an error! Skipping...")
      if (token[4] !== Tree.empty) token[4] = Tree.empty
      return true
    }
  }

  /**
   * Advances the oldest parser, and then returns whether or not the
   * handler has finished with this step.
   *
   * @param force - Forces parsers to return a tree, either through
   *   advancing them fully or by calling their `forceFinish` method.
   */
  advance(force = false) {
    if (this.done) return true
    const parser = this.parsers[0]
    const finished = this.advanceParser(parser, force)

    if (finished) this.parsers.shift()

    return this.done
  }

  /** Serializes the handler's state. */
  serialize(): EmbeddedData {
    const pending = [...this.pending]
    const parsers: EmbeddedData["parsers"] = []

    // this function gets called a lot, so we're doing the verbose method
    // for loops are far faster than the `map` function, unfortunately
    for (let idx = 0; idx < this.parsers.length; idx++) {
      // prettier-ignore
      const { index, lang: { range: [lang, start, end] } } = this.parsers[idx];
      parsers[idx] = [index, [lang, start, end]]
    }

    return { pending, parsers }
  }
}
