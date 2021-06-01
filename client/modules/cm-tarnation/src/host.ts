import { EditorParseContext } from "@codemirror/language"
import { Input, PartialParse, Tree } from "lezer-tree"
import { isEmpty, perfy } from "wj-util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserBuffer, ParserContext, ParserStack } from "./parser"
import { Tokenizer, TokenizerBuffer, TokenizerContext, TokenizerStack } from "./tokenizer"
import type { ParseRegion } from "./types"

/** Current stage of the host. */
enum Stage {
  Tokenize,
  Parse
}

/**
 * The `Host` object is the "parser" that CodeMirror interacts with to
 * build a syntax tree. In reality, the `Host` is well, a host, for a
 * separate `Tokenizer` and `Parser`. These are ran in stages - first the
 * tokenizer, and then the parser.
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
  /**
   * The starting position. This position isn't exactly the start of the
   * region that should be parsed, instead it is the start of the
   * *document* that is being parsed. This is because it may be the case
   * that the `Host` is actually being embedded, and is only acting on a
   * small part of a larger document.
   */
  private declare start: number

  /** The current `Stage`, either tokenizing or parsing. */
  private declare stage: Stage

  /**
   * An object storing details about the region of the document to be
   * parsed, where it was edited, the length, etc.
   */
  private declare region: ParseRegion

  /**
   * CodeMirror's context object. This isn't actually required, but it
   * allows for much easier usage of incremental parsing.
   */
  private declare context?: EditorParseContext

  /** The editor viewport, as in what range of text can the user actually see. */
  private declare viewport?: { from: number; to: number }

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
   * @param start - The starting position of the document.
   * @param context - A CodeMirror `EditorParseContext`, if available.
   */
  constructor(
    language: TarnationLanguage,
    input: Input,
    start: number,
    context?: EditorParseContext
  ) {
    // check for bogus contexts (e.g. `{}`)
    if (context && !(context instanceof EditorParseContext) && isEmpty(context)) {
      context = undefined
    }

    this.language = language
    this.input = input
    this.start = start
    this.stage = Stage.Tokenize
    this.context = context

    this.measurePerformance = perfy()

    // get edited region
    if (context?.fragments?.length) {
      const fragments = context.fragments
      const firstFragment = fragments[0]
      const lastFragment = fragments[fragments.length - 1]

      if (fragments.length === 1) {
        this.region = {
          from: start,
          // to: firstFragment.from,
          to: input.length,
          length: input.length,
          edit: {
            from: start,
            to: firstFragment.from,
            offset: -firstFragment.offset
          }
        }
      } else {
        this.region = {
          from: Math.max(firstFragment.to, start),
          // to: lastFragment.from,
          to: input.length,
          length: input.length,
          edit: {
            from: firstFragment.to,
            to: lastFragment.from,
            offset: -lastFragment.offset
          }
        }
      }

      if (context.viewport && context.skipUntilInView!) {
        this.viewport = context.viewport

        const v = context.viewport
        const r = this.region

        // basically doubles the height of the viewport
        // this adds a bit of a buffer between the actual end and the end of parsing
        // otherwise if you scrolled too fast you'd see unparsed sections easily
        const end = v.to + (v.to - v.from)

        if (v.from < r.to && r.to > end) r.to = end
      }
    } else {
      this.region = {
        from: start,
        to: input.length,
        length: input.length
      }
    }

    // find cached data, if possible
    if (context?.fragments?.length) {
      for (let idx = 0; idx < context.fragments.length; idx++) {
        const f = context.fragments[idx]
        // make sure fragment is within the region of the document we care about
        if (f.from > start || f.to < start) continue

        // try to find the buffer for this fragment's tree in the cache
        const buffer = this.language.cache.find(f.tree, start, f.to)
        if (buffer) {
          // try to find a suitable chunk from the buffer to restart the tokenizer from
          const { chunk, index } = buffer.search(this.region.from, -1)
          if (chunk && index !== null) {
            // split the buffer, reuse the left side,
            // but keep the right side around for reuse as well
            const { left, right } = buffer.split(index)
            this.tokenizerPreviousRight = right
            this.region.from = chunk.context.pos
            this.setupTokenizer(left, chunk.context)

            // check if parser has a cached state for this chunk
            if (this.language.cache.has(chunk)) {
              const context = ParserContext.deserialize(this.language.cache.get(chunk)!)
              this.setupParser(context)
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
    if (!context) {
      context = new ParserContext(this.start, 0, new ParserBuffer(), new ParserStack(), {
        pending: [],
        parsers: []
      })
    }

    this.parser = new Parser(
      this.language,
      context,
      this.input,
      this.region,
      [],
      this.context
    )
  }

  /**
   * The current "position" of the host. This isn't really all that
   * accurate, as it's only reporting the tokenizer's position. That means
   * when the parser is running, the position will just be sitting still.
   */
  get pos() {
    return this.tokenizer.context.pos
  }

  /** Advances the tokenizer or parser one step, depending on the current stage. */
  advance(): Tree | null {
    if (!this.measurePerformance) this.measurePerformance = perfy()
    switch (this.stage) {
      case Stage.Tokenize: {
        // try to reuse ahead state
        const reused =
          this.tokenizerPreviousRight &&
          this.tokenizer.tryToReuse(this.tokenizerPreviousRight)

        // can't reuse the buffer more than once (pointless)
        if (reused) this.tokenizerPreviousRight = undefined

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
        const result = this.parser.advance()
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
    const length = this.pos - this.start

    const tree = Tree.build({
      topID: 0,
      nodeSet: this.language.nodes!.set,
      buffer,
      reused,
      length,
      start: this.start
    })

    this.language.cache.attach(this.tokenizer.buffer, tree)

    if (this.context?.skipUntilInView && length < this.region.length) {
      this.context.skipUntilInView(this.pos, this.region.length)
    }

    if (this.measurePerformance) {
      this.performance = this.measurePerformance()
      this.measurePerformance = undefined
      this.language.performance = this.performance
    }

    return tree
  }

  /**
   * Forces a `Tree` to be assembled from the tokenizer and parser state
   * *right now*, regardless of anything that might be incomplete or just
   * simply not started. In the parse stage, this is relatively simple, but
   * in the tokenizer stage, this is more complex.
   *
   * Calling `forceFinish` during tokenization means that the tokenizer
   * will immediately have its incomplete chunks sent to the parser, which
   * will then be *fully advanced*. Embedded languages will also have their
   * parser's `forceFinish` method called.
   */
  forceFinish(): Tree {
    switch (this.stage) {
      case Stage.Tokenize: {
        this.parser.pending = this.tokenizer.chunks
        const { buffer, reused } = this.parser.forceFinish()
        return this.finish(buffer, reused)
      }
      case Stage.Parse: {
        const { buffer, reused } = this.parser.forceFinish()
        return this.finish(buffer, reused)
      }
    }
  }
}
