import {
  Input,
  NestedParse,
  parseMixed,
  Parser as CodeMirrorParser,
  ParseWrapper,
  PartialParse,
  Tree,
  TreeCursor,
  TreeFragment
} from "@lezer/common"
import { LanguageDescription, ParseContext } from "@wikijump/codemirror/cm"
import { perfy } from "@wikijump/util"
import { ChunkBuffer } from "./chunk/buffer"
import { compileChunks } from "./chunk/parsing"
import type { GrammarState } from "./grammar/state"
import type { TarnationLanguage } from "./language"
import { ParseRegion } from "./region"
import type { GrammarToken } from "./types"
import { canContinue, EmbeddedParserProp } from "./util"

const DISABLED_NESTED = true
const REUSE_LEFT = true
const REUSE_RIGHT = true
const MARGIN_BEFORE = 32
const MARGIN_AFTER = 128

/**
 * Factory for correctly instantiating {@link Parser} instances. To
 * CodeMirror, this class is the `parser`, and a {@link Parser} is the
 * running process of said parser.
 */
export class ParserFactory extends CodeMirrorParser {
  /** A wrapper function that enables mixed parsing. */
  private declare wrapper: ParseWrapper

  /**
   * @param language - The {@link TarnationLanguage} that this factory
   *   passes to the {@link Parser} instances it constructs.
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
    const parser = new Parser(this.language, input, fragments, ranges)
    return DISABLED_NESTED ? parser : this.wrapper(parser, input, fragments, ranges)
  }

  /**
   * Special "nest" function provided to the `parseMixed` function.
   * Determines which nodes indicate a nested parsing region, and if so,
   * returns a `NestedParser` for said region.
   */
  private nest(node: TreeCursor, input: Input): NestedParse | null {
    if (node.type.prop(EmbeddedParserProp)) {
      // get name from the per-node property
      const name = node.type.prop(EmbeddedParserProp)
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

/**
 * The `Parser` is the main interface for tokenizing and parsing, and what
 * CodeMirror directly interacts with.
 *
 * Additionally, the `Parser` handles the recovery of grammar state from
 * the stale trees provided by CodeMirror, and then uses this data to
 * restart tokenization with reused tokens.
 *
 * Note that the `Parser` is not persistent a objects It is discarded as
 * soon as the parse is done. That means that its startup time is very significant.
 */
export class Parser implements PartialParse {
  /** The host language. */
  private declare language: TarnationLanguage

  /**
   * An object storing details about the region of the document to be
   * parsed, where it was edited, the length, etc.
   */
  private declare region: ParseRegion

  /** The current state of the grammar, such as the stack. */
  private declare state: GrammarState

  /** {@link Chunk} buffer, where matched tokens are cached. */
  private declare buffer: ChunkBuffer

  /**
   * A buffer containing the stale *ahead* state of the tokenized output.
   * As in, when a user makes a change, this is all of the tokenization
   * data for the previous document after the location of that new change.
   */
  private declare previousRight?: ChunkBuffer

  /** A function used to measure how long the parse is taking. */
  private declare measurePerformance: (msg?: string) => number

  /** The current position of the parser. */
  declare parsedPos: number

  /**
   * The position the parser will be stopping at early, if given a location
   * to stop at.
   */
  declare stoppedAt: number | null

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
    this.region = new ParseRegion(input, ranges, fragments)
    this.stoppedAt = null

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
          // try to find a suitable chunk from the buffer to restart tokenization from
          const { chunk, index } = buffer.search(this.region.edit!.from, -1)
          if (chunk && index !== null) {
            // split the buffer, reuse the left side,
            // but keep the right side around for reuse as well
            const { left, right } = buffer.split(index)
            this.previousRight = right
            this.region.from = chunk.pos
            this.buffer = left
            this.state = chunk.state.clone()
          }
        }
      }
    }

    this.parsedPos = this.region.from

    // if we couldn't reuse state, we'll need to startup things with a default state
    if (!this.buffer || !this.state) {
      this.buffer = new ChunkBuffer()
      this.state = this.language.grammar!.startState()
    }
  }

  /** True if the parser is done. */
  get done() {
    return this.parsedPos >= this.region.to
  }

  /**
   * Notifies the parser to not progress past the given position.
   *
   * @param pos - The position to stop at.
   */
  stopAt(pos: number) {
    this.region.to = pos
    this.stoppedAt = pos
  }

  /** Advances tokenization one step. */
  advance(): Tree | null {
    // if we're told to stop, we need to BAIL
    if (this.stoppedAt && this.parsedPos >= this.stoppedAt) {
      return this.finish()
    }

    this.nextChunk()

    if (this.done) return this.finish()

    return null
  }

  private finish(): Tree {
    const cursor = compileChunks(this.buffer.chunks)

    const start = this.region.original.from
    const length = this.region.original.length
    const nodeSet = this.language.nodeSet!

    // build tree from buffer
    const tree = Tree.build({
      topID: this.language.top!.id,
      buffer: cursor,
      nodeSet,
      start,
      length
    })

    // bit of a hack (private properties)
    // this is so that we don't need to build another tree
    const props = Object.create(null)
    // @ts-ignore
    props[this.language.stateProp!.id] = this.buffer
    // @ts-ignore
    tree.props = props

    const context = ParseContext.get()

    // inform editor that we skipped everything past the viewport
    if (
      context &&
      !this.stoppedAt &&
      this.parsedPos > context.viewport.to &&
      this.parsedPos < this.region.original.to
    ) {
      context.skipUntilInView(this.parsedPos, this.region.original.to)
    }

    this.performance = this.measurePerformance()
    this.language.performance = this.performance

    return tree
  }

  /** Advances the parser to the next chunk. */
  private nextChunk() {
    // this condition is a little misleading,
    // as we're actually going to break out when any chunk is emitted.
    // however, if we're at the "last chunk", this condition catching that
    while (this.parsedPos < this.region.to) {
      if (REUSE_RIGHT) {
        // try to reuse ahead state
        const reused = this.previousRight && this.tryToReuse(this.previousRight)
        // can't reuse the buffer more than once (pointless)
        if (reused) this.previousRight = undefined
      }

      const pos = this.parsedPos
      const startState = this.state.clone()

      let matchTokens: GrammarToken[] | null = null
      let length = 1

      const start = Math.max(pos - MARGIN_BEFORE, this.region.from)
      const startCompensated = this.region.compensate(pos, start - pos)

      const str = this.region.read(startCompensated, MARGIN_AFTER, this.region.to)

      const match = this.language.grammar!.match(this.state, str, pos - start, pos)

      if (match) {
        this.state = match.state
        matchTokens = match.compile()
        length = match.length || 1
      }

      this.parsedPos = this.region.compensate(pos, length)

      const tokens: GrammarToken[] = []

      if (matchTokens?.length) {
        let last!: GrammarToken

        for (let idx = 0; idx < matchTokens.length; idx++) {
          const t = matchTokens[idx]

          if (!t[0] && !t[3] && !t[4]) continue

          if (!this.region.contiguous) {
            const from = this.region.compensate(pos, t[1] - pos)
            const end = this.region.compensate(pos, t[2] - pos)
            t[1] = from
            t[2] = end
          }

          // check if the new token can be merged into the last one
          if (last && canContinue(last, t)) last[2] = t[2]
          else tokens.push((last = t))
        }
      }

      if (tokens?.length) {
        // if a chunk was added we'll break out of the loop
        if (this.buffer.add(pos, startState, tokens)) return true
      }
    }

    return false
  }

  /**
   * Tries to reuse a buffer *ahead* of the current position. Returns true
   * if this was successful, otherwise false.
   *
   * @param right - The buffer to try and reuse.
   */
  private tryToReuse(right: ChunkBuffer) {
    // can't reuse if we don't know the safe regions
    if (!this.region.edit) return false
    // can only safely reuse if we're ahead of the edited region
    if (this.parsedPos <= this.region.edit.to) return false

    // check every chunk and see if we can reuse it
    for (let idx = 0; idx < right.chunks.length; idx++) {
      const chunk = right.chunks[idx]
      if (chunk.isReusable(this.state, this.parsedPos, this.region.edit.offset)) {
        right.slide(idx, this.region.edit.offset, true)
        this.buffer.link(right, this.region.original.length)
        this.buffer.ensureLast(this.parsedPos, this.state)
        this.state = this.buffer.last!.state.clone()
        this.parsedPos = this.buffer.last!.pos
        return true
      }
    }

    return false
  }

  /**
   * Returns the first chunk buffer found within a tree, if any.
   *
   * @param tree - The tree to search through, recursively.
   * @param from - The start of the search area.
   * @param to - The end of the search area.
   * @param offset - An offset added to the tree's positions, so that they
   *   may match some other source's positions.
   */
  private find(tree: Tree, from: number, to: number, offset = 0): ChunkBuffer | null {
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
}
