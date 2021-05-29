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
    const embedded = this.context.embedded
    this.pending = [...embedded.pending]
    this.parsers = embedded.parsers.map(([index, range]) => ({
      lang: new EmbeddedLanguage(this.language, range),
      index
    }))
  }

  /** Whether or not the handler has fully completed parsing. */
  get done() {
    return this.parsers.length === 0
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
        throw new Error("Attempted to push an unassigned language!")
      }
      const index = this.pending.shift()!
      const lang = new EmbeddedLanguage(this.language, embed as EmbedToken)
      this.parsers.push({ index, lang })
    }
  }

  /** Advances the oldest parser. */
  advance() {
    if (this.done) return true
    const { parsers, editorContext } = this
    const { index, lang } = parsers[0]
    const [, start, end] = lang.range
    const token = this.context.buffer.get(index)!

    if (token[4] !== Tree.empty && this.context.start >= end) {
      parsers.shift()
      if (this.done) return true
      return null
    }

    const parser = (parsers[0].parser ||= lang.startParse(
      this.input.clip(end),
      start,
      editorContext
    ))

    // effectively marks the tree as stale
    if (token[4] !== Tree.empty) token[4] = Tree.empty

    const done = parser.advance()
    if (done) {
      token[0] = -1
      token[4] = done
      parsers.shift()
      if (this.done) return true
    }
    return null
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
