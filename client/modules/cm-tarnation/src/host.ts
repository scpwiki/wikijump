import {
  Input,
  NestedParse,
  parseMixed,
  Parser as ParserBase,
  ParseWrapper,
  PartialParse,
  Tree,
  TreeCursor,
  TreeFragment
} from "@lezer/common"
import { LanguageDescription, ParseContext } from "@wikijump/codemirror/cm"
import { perfy } from "@wikijump/util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserContext } from "./parser"
import { ParseRegion } from "./region"
import { Tokenizer, TokenizerBuffer, TokenizerContext } from "./tokenizer"
import { EmbeddedParserProp, EmbeddedParserType } from "./util"

const BAIL = false
const SKIP_PARSER = false
const REUSE_LEFT = true
const REUSE_RIGHT = true

/**
 * Factory for correctly instantiating {@link Host} instances. To
 * CodeMirror, this class is the `parser`, and a {@link Host} is the running
 * process of said parser.
 */
export class HostFactory extends ParserBase {
  /** The wrapper function that enables mixed parsing. */
  private declare wrapper: ParseWrapper

  /**
   * @param language - The {@link TarnationLanguage} that this factory
   *   passes to the {@link Host} instances it constructs.
   */
  constructor(private language: TarnationLanguage) {
    super()
    this.wrapper = parseMixed(this.nest.bind(this))
  }

  createParse(
    input: Input,
    fragments: TreeFragment[],
    ranges: { from: number; to: number }[]
  ) {
    const delegator = new Host(this.language, input, fragments, ranges)
    return this.wrapper(delegator, input, fragments, ranges)
  }

  /**
   * Special "nest" function provided to the `parseMixed` function.
   * Determines which nodes indicate a nested parsing region, and if so,
   * returns a `NestedParser` for said region.
   */
  private nest(node: TreeCursor, input: Input): NestedParse | null {
    if (node.type === EmbeddedParserType && node.tree) {
      // get name from the per-node property
      const name = node.tree.prop(EmbeddedParserProp)
      if (!name) return null

      // don't bother with empty nodes
      if (node.from === node.to) return null

      let langs: readonly LanguageDescription[]

      if (!(this.language.nestLanguages instanceof Array)) {
        const context = ParseContext.get()
        langs = context ? context.state.facet(this.language.nestLanguages) : []
      } else {
        langs = this.language.nestLanguages
      }

      const lang = LanguageDescription.matchLanguageName(langs, name)

      // language doesn't exist
      if (!lang) return null

      // language already loaded
      if (lang.support) {
        return {
          parser: lang.support.language.parser,
          overlay: [{ from: node.from, to: node.to }]
        }
      }

      // language not loaded yet
      return {
        parser: ParseContext.getSkippingParser(lang.load()),
        overlay: [{ from: node.from, to: node.to }]
      }
    }

    return null
  }
}

/** Current stage of the host. */
enum Stage {
  Tokenize,
  Parse
}

/**
 * The host is the main interface between the parser and the tokenizer, and
 * what CodeMirror directly interacts with when parsing.
 *
 * Additionally, the `Host` handles the recovery of tokenizer and parser
 * states from the stale trees provided by CodeMirror, and then uses this
 * data to restart the tokenizer and parser with reused state.
 *
 * Note that the `Host`, along with the `Tokenizer` and `Parser`, are not
 * persistent objects. They are discarded as soon as the parse is done.
 * That means that their startup time is very significant.
 */
export class Host implements PartialParse {
  /** The host language. */
  private declare language: TarnationLanguage

  /** The input document to parse. */
  private declare input: Input

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
   * @param ranges - The ranges of the document to parse.
   */
  constructor(
    language: TarnationLanguage,
    input: Input,
    fragments: TreeFragment[],
    ranges: { from: number; to: number }[]
  ) {
    this.measurePerformance = perfy()

    this.language = language
    this.input = input
    this.stage = Stage.Tokenize

    this.region = new ParseRegion(input, ranges, fragments)

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

            // find latest possible parser context
            let chunkIndex: number | null = null
            for (let i = 0; i < left.buffer.length; i++) {
              if (left.buffer[i].hasParserContext) chunkIndex = i
            }

            if (chunkIndex) {
              this.setupParser(left.buffer[chunkIndex].parserContext!)
            }
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
      context = new TokenizerContext(
        this.region.from,
        this.language.grammar!.startState()
      )
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
    if (!context) context = new ParserContext()

    this.parser = new Parser(this.language, context, this.region)
  }

  /**
   * The current "position" of the host. This isn't really all that
   * accurate, as it's only reporting the tokenizer's position. That means
   * when the parser is running, the position will just be sitting still.
   */
  get parsedPos() {
    return this.tokenizer.context.pos
  }

  /**
   * The position the parser will be stopping at early, if given a location
   * to stop at.
   */
  stoppedAt: number | null = null

  /**
   * Notifies the parser to not progress past the given position.
   *
   * @param pos - The position to stop at.
   */
  stopAt(pos: number) {
    this.stoppedAt = pos
    this.region.to = pos
  }

  /** Advances the tokenizer or parser one step, depending on the current stage. */
  advance(): Tree | null {
    if (!this.measurePerformance) this.measurePerformance = perfy()

    // if we're overbudget, BAIL
    if (BAIL && this.stoppedAt && this.measurePerformance() >= 12) {
      this.parser.pending = this.tokenizer.chunks
      if (this.tokenizer.chunks.length > this.parser.context.index) {
        // forced to reparse, didn't get up to the point where we reused
        this.parser.context = new ParserContext()
      }
      const { buffer, reused } = this.parser.parseFully()
      return this.finish(buffer, reused)
    }

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
          : this.tokenizer.tokenize()

        if (chunks) {
          this.parser.pending = chunks
          this.stage = Stage.Parse
        }

        return null
      }
      case Stage.Parse: {
        const result = SKIP_PARSER ? this.parser.parseFullyRaw() : this.parser.parse()

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
    const start = this.region.original.from
    const length = this.parsedPos - this.region.original.from
    const nodeSet = this.language.nodeSet!

    // build tree from buffer
    const built = Tree.build({ topID: 0, buffer, nodeSet, reused, start })

    // wrap built children in a tree with the buffer cached
    const tree = new Tree(this.language.top!, built.children, built.positions, length, [
      [this.language.stateProp!, this.tokenizer.buffer]
    ])

    const context = ParseContext.get()

    // inform editor that we skipped everything past the viewport
    if (
      context &&
      this.parsedPos > context.viewport.to &&
      this.parsedPos < this.region.original.to
    ) {
      context.skipUntilInView(this.parsedPos, this.region.original.to)
    }

    if (this.measurePerformance) {
      this.performance = this.measurePerformance()
      this.measurePerformance = undefined
      this.language.performance = this.performance
    }

    return tree
  }
}
