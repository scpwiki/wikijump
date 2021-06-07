import { Input, PartialParse, Tree } from "lezer-tree"
import type { EditorParseContext } from "wj-codemirror/cm"
import type { TarnationLanguage } from "../language"
import type { EmbeddedData, LezerToken } from "../types"
import type { ParserContext } from "./context"
import { EmbeddedLanguage } from "./embedded-language"

/**
 * An `EmbeddedHandler` object is a handler that creates a dramatically
 * simpler interface for managing embedded regions of other languages in a
 * Tarnation parse tree.
 *
 * Every time the `advance` method is called, the oldest parser is advanced
 * one step. Once it is done, the resultant tree is inserted into the
 * token, and the parser is removed from the list. The cycle continues
 * until every parser is complete.
 */
export class EmbeddedHandler {
  /** List of pending embedded regions to process. */
  private embeds: {
    /** The token the final tree will be emitted to. */
    token: LezerToken
    /** The embedded language that is being used for parsing. */
    lang: EmbeddedLanguage
    /** The parse itself, if it has been started. */
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
    return this.embeds.length === 0
  }

  /** Reset's the internal state to match the given context. */
  set(context: ParserContext) {
    this.context = context
    this.embeds = []
    for (let idx = 0; idx < context.embedded.length; idx++) {
      this.embeds[idx] = {
        token: context.embedded[idx][0],
        lang: new EmbeddedLanguage(this.language, context.embedded[idx][1])
      }
    }
  }

  /**
   * Adds a new embedded region to parse.
   *
   * @param token - The token which represents the region.
   * @param language - The language to embed with.
   */
  add(token: LezerToken, language: string) {
    this.embeds.push({ token, lang: new EmbeddedLanguage(this.language, language) })
  }

  /**
   * Advances a parser. Returns whether or not the parser given has finished.
   *
   * @param embed - The embedded parser to advance.
   * @param force - Whether or not the parser should be forced to return a tree.
   */
  private advanceParser(
    embed: { token: LezerToken; lang: EmbeddedLanguage; parser?: PartialParse },
    force = false
  ) {
    const { token, lang } = embed
    const [, from, to] = token

    if (!force && !lang.ready) return false
    if (token[4] !== Tree.empty && this.context.start >= to) return true

    try {
      let tree: Tree | null = null
      let parser: PartialParse

      if (!embed.parser) {
        parser = lang.startParse(this.input.clip(to), from, this.editorContext)
        embed.parser = parser
      } else {
        parser = embed.parser
      }

      if (!tree) {
        // effectively marks the tree as stale
        if (token[4] !== Tree.empty) token[4] = Tree.empty

        if (force) {
          const viewport = this.editorContext?.viewport
          // check if the region is visible, if it is, we're going to advance it fully
          // if not, we'll just try a forceFinish
          if (viewport && to > viewport.from && from < viewport.to) {
            while ((tree = parser.advance()) === null) {}
          } else {
            tree = parser.forceFinish()
          }
        } else {
          tree = parser.advance()
        }
      }

      if (tree) {
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
    const parser = this.embeds[0]
    const finished = this.advanceParser(parser, force)

    if (finished) this.embeds.shift()

    return this.done
  }

  /** Serializes the handler's state. */
  serialize(): EmbeddedData {
    // due to the way this gets used, we only need a shallow clone.
    // if we did a full clone, it wouldn't really work
    const serialized: EmbeddedData = []
    for (let idx = 0; idx < this.embeds.length; idx++) {
      serialized[idx] = [this.embeds[idx].token, this.embeds[idx].lang.language]
    }
    return serialized
  }
}
