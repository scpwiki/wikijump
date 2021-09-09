import { Input, NodeType, Tree, TreeFragment } from "@lezer/common"
import { ParseContext } from "modules/wj-codemirror/cm"
import { perfy } from "wj-util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserContext } from "./parser"
import { ParseRegion } from "./region"
import { Tokenizer, TokenizerBuffer, TokenizerContext, TokenizerStack } from "./tokenizer"

const SKIP_PARSER = false
const REUSE_LEFT = true
const REUSE_RIGHT = true

/** Current stage of the host. */
enum Stage {
  Tokenize,
  Parse
}

/**
 * The host is the main interface between the parser and the tokenizer,
 * created for each range given to the {@link Delegator}. It is effectively
 * the actual "parser", but due to the potentially non-contiguous nature of
 * the input, a host is created for each range.
 *
 * Additionally, the `Host` handles the recovery of tokenizer and parser
 * states from the stale trees provided by CodeMirror, and then uses this
 * data to restart the tokenizer and parser with reused state.
 *
 * Note that the `Host`, along with the `Tokenizer` and `Parser`, are not
 * persistent objects. They are discarded as soon as the parse is done.
 * That means that their startup time is very significant.
 */
export class Host {
  /** The host language. */
  private declare language: TarnationLanguage

  /** The input document to parse. */
  private declare input: Input

  /**
   * If true, the host should return a `Tree` with the language's top level
   * `NodeType`.
   */
  private declare top: boolean

  /** The current `Stage`, either tokenizing or parsing. */
  declare stage: Stage

  /**
   * An object storing details about the region of the document to be
   * parsed, where it was edited, the length, etc.
   */
  declare region: ParseRegion

  /** The tokenizer to be used. */
  private declare tokenizer: Tokenizer

  /**
   * A buffer containing the previous *ahead* state of the tokenizer's
   * output. As in, when a user makes a change, this is all of the
   * tokenization data for the previous document after the location of that
   * new change.
   */
  private declare tokenizerPreviousRight?: TokenizerBuffer

  /** The parser to be used. */
  private declare parser: Parser

  /** A function used to measure how long the parse is taking. */
  private declare measurePerformance?: (msg?: string) => number

  /** The current performance value, in milliseconds. */
  declare performance?: number

  /**
   * @param language - The language containing the grammar to use.
   * @param input - The input document to parse.
   * @param fragments - The fragments to be used for determining reuse of
   *   previous parses.
   * @param range - The range of the document to parse.
   * @param top - If true, the host will return a `Tree` with the
   *   language's top level `NodeType`.
   */
  constructor(
    language: TarnationLanguage,
    input: Input,
    fragments: TreeFragment[],
    range: { from: number; to: number },
    top: boolean
  ) {
    this.measurePerformance = perfy()

    this.language = language
    this.input = input
    this.stage = Stage.Tokenize
    this.top = top

    this.region = new ParseRegion(
      { from: range.from, to: input.length },
      { from: range.from, to: range.to },
      fragments
    )

    const context = ParseContext.get()

    // parse only the viewport
    if (context?.viewport && context.skipUntilInView!) {
      const v = context.viewport
      const r = this.region

      // basically doubles the height of the viewport
      // this adds a bit of a buffer between the actual end and the end of parsing
      // otherwise if you scrolled too fast you'd see unparsed sections easily
      const end = v.to + (v.to - v.from)

      if (v.from < r.to && r.to > end) r.to = end
    }

    // find cached data, if possible
    if (REUSE_LEFT && fragments?.length) {
      for (let idx = 0; idx < fragments.length; idx++) {
        const f = fragments[idx]
        // make sure fragment is within the region of the document we care about
        if (f.from > this.region.from || f.to < this.region.from) continue

        // try to find the buffer for this fragment's tree in the cache
        const buffer = this.find(f.tree, this.region.from, f.to)

        if (buffer) {
          // try to find a suitable chunk from the buffer to restart the tokenizer from
          const { chunk, index } = buffer.search(this.region.edit!.from, -1)
          if (chunk && index !== null) {
            // split the buffer, reuse the left side,
            // but keep the right side around for reuse as well
            const { left, right } = buffer.split(index)
            this.tokenizerPreviousRight = right
            this.region.from = chunk.context.pos
            this.setupTokenizer(left, chunk.context)

            // check if parser has a cached state for this chunk
            const context = chunk.parserContext
            if (context) this.setupParser(context)
          }
        }
      }
    }

    // if we couldn't reuse state, we'll need to startup things with a default state
    if (!this.tokenizer) this.setupTokenizer()
    if (!this.parser) this.setupParser()
  }

  /**
   * Returns the first tokenizer buffer found within a tree, if any.
   *
   * @param tree - The tree to search through, recursively.
   * @param from - The start of the search area.
   * @param to - The end of the search area.
   * @param offset - An offset added to the tree's positions, so that they
   *   may match some other source's positions.
   */
  private find(tree: Tree, from: number, to: number, offset = 0): TokenizerBuffer | null {
    const bundle =
      offset >= from && offset + tree.length >= to
        ? tree.prop(this.language.stateProp!)
        : undefined

    if (bundle) return bundle

    // recursively check children
    for (let i = tree.children.length - 1; i >= 0; i--) {
      const child = tree.children[i]
      const pos = offset + tree.positions[i]
      if (!(child instanceof Tree && pos < to)) continue
      const found = this.find(child, from, to, pos)
      if (found) return found
    }

    return null
  }

  /**
   * Instantiates the `Tokenizer`.
   *
   * @param buffer - A `TokenizerBuffer` to reuse.
   * @param context - A `TokenizerContext` to reuse.
   */
  private setupTokenizer(buffer?: TokenizerBuffer, context?: TokenizerContext) {
    if (!buffer || !context) {
      const stack = new TokenizerStack({ stack: [["root", {}]], embedded: null })
      context = new TokenizerContext(this.region.from, stack)
      buffer = new TokenizerBuffer()
    }

    this.tokenizer = new Tokenizer(
      this.language,
      context,
      buffer,
      this.input,
      this.region
    )
  }

  /**
   * Instantiates the `Parser`.
   *
   * @param context - A `ParserContext` to reuse.
   */
  private setupParser(context?: ParserContext) {
    if (!context) context = new ParserContext(this.region.from)

    this.parser = new Parser(this.language, context, this.region)
  }

  /**
   * The current "position" of the host. This isn't really all that
   * accurate, as it's only reporting the tokenizer's position. That means
   * when the parser is running, the position will just be sitting still.
   */
  get pos() {
    return this.tokenizer.context.pos
  }

  /**
   * Notifies the parser to not progress past the given position.
   *
   * @param pos - The position to stop at.
   */
  stopAt(pos: number) {
    this.region.to = pos
  }

  /** Advances the tokenizer or parser one step, depending on the current stage. */
  advance(): Tree | null {
    if (!this.measurePerformance) this.measurePerformance = perfy()
    switch (this.stage) {
      case Stage.Tokenize: {
        if (REUSE_RIGHT) {
          // try to reuse ahead state
          const reused =
            this.tokenizerPreviousRight &&
            this.tokenizer.tryToReuse(this.tokenizerPreviousRight)

          // can't reuse the buffer more than once (pointless)
          if (reused) this.tokenizerPreviousRight = undefined
        }

        // try an advance if we're not done
        const chunks = this.tokenizer.done
          ? this.tokenizer.chunks
          : this.tokenizer.advance()

        if (chunks) {
          this.parser.pending = chunks
          this.stage = Stage.Parse
        }

        return null
      }
      case Stage.Parse: {
        const result = SKIP_PARSER ? this.parser.advanceFullyRaw() : this.parser.advance()

        if (result) {
          const { buffer, reused } = result
          return this.finish(buffer, reused)
        }

        return null
      }
    }
  }

  /**
   * Returns a `Tree` given a finalized buffer and reused `Tree` nodes.
   * This also performs caching using the `Tree` as a key, along with some
   * other housekeeping.
   *
   * @param buffer - The `Tree.build` buffer to use.
   * @param reused - A list of reused tree nodes whose indexes are
   *   referenced in the buffer.
   */
  private finish(buffer: number[], reused: Tree[]): Tree {
    const top = this.top ? this.language.top! : NodeType.none
    const start = this.region.original.from
    const length = this.pos - this.region.original.from
    const nodeSet = this.language.nodes!.set

    // build tree from buffer
    const built = Tree.build({ topID: 0, buffer, nodeSet, reused, start })

    // wrap built children in a tree with the buffer cached
    const tree = new Tree(top, built.children, built.positions, length, [
      [this.language.stateProp!, this.tokenizer.buffer]
    ])

    const context = ParseContext.get()

    // inform editor that we skipped everything past the viewport
    if (
      context?.skipUntilInView &&
      this.pos > context.viewport.to &&
      this.pos < this.region.original.to
    ) {
      context.skipUntilInView(this.pos, this.region.original.to)
    }

    if (this.measurePerformance) {
      this.performance = this.measurePerformance()
      this.measurePerformance = undefined
      this.language.performance = this.performance
    }

    return tree
  }
}
