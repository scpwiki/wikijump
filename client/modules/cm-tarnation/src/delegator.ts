import {
  Input,
  parseMixed,
  Parser,
  ParseWrapper,
  PartialParse,
  Tree,
  TreeCursor,
  TreeFragment
} from "@lezer/common"
import { LanguageDescription, ParseContext } from "@wikijump/codemirror/cm"
import { EmbeddedParserProp, EmbeddedParserType } from "."
import { Host } from "./host"
import { TarnationLanguage } from "./language"

// TODO: figure out if this non-contiguous approach is correct

/**
 * A "parser" that serves as a factory for the {@link Delegator}
 * `PartialParse` class. This class also wraps the constructed delegators
 * with a mixed parser so that nested parsing works.
 */
export class DelegatorFactory extends Parser {
  /** The wrapper function that enables mixed parsing. */
  private declare wrapper: ParseWrapper

  /**
   * @param language - The {@link TarnationLanguage} that this factory
   *   passes to the {@link Delegator} instances it constructs.
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
    const delegator = new Delegator(this.language, input, fragments, ranges)
    return this.wrapper(delegator, input, fragments, ranges)
  }

  /**
   * Special "nest" function provided to the `parseMixed` function.
   * Determines which nodes indicate a nested parsing region, and if so,
   * returns a `NestedParser` for said region.
   */
  private nest(node: TreeCursor, input: Input) {
    if (node.type === EmbeddedParserType && node.tree) {
      // get name from the per-node property
      const name = node.tree.prop(EmbeddedParserProp)
      if (!name) return null

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
      if (lang.support) return { parser: lang.support.language.parser }
      // language not loaded yet
      return { parser: ParseContext.getSkippingParser(lang.load()) }
    }

    return null
  }
}

/**
 * The `Delegator` object implements the `PartialParse` interface but
 * delegates the actual parsing to various {@link Host} instances,
 * specifically a `Host` is created for each range provided to the delegator.
 */
export class Delegator implements PartialParse {
  /** The current range index. */
  private declare index: number

  /** The host being advanced. */
  private declare host: Host

  /** The list of already completed `Tree`s. */
  private declare trees: Tree[]

  /**
   * @param language - The {@link TarnationLanguage} to retrieve data from.
   * @param input - The input to parse.
   * @param fragments - The fragments to be used for determining reuse of
   *   previous parses.
   * @param ranges - The document ranges to be parsed.
   */
  constructor(
    private language: TarnationLanguage,
    private input: Input,
    private fragments: TreeFragment[],
    private ranges: { from: number; to: number }[]
  ) {
    this.index = 0
    this.trees = []
    this.host = this.hostOf(this.index)
  }

  /** Returns a {@link Host} instance for the specified range index. */
  private hostOf(index: number) {
    return new Host(
      this.language,
      this.input,
      this.fragments,
      this.ranges[index],
      this.ranges.length <= 1
    )
  }

  /**
   * Advances the parser(s) one step, and returns a `Tree` if done. Returns
   * `null` otherwise.
   */
  advance() {
    const tree = this.host.advance()

    if (tree) {
      this.trees.push(tree)

      if (this.ranges.length === 1) {
        return this.finish()
      }

      if (this.index < this.ranges.length - 1) {
        if (this.ranges[this.index].from >= (this.stoppedAt ?? this.input.length)) {
          return this.finish()
        } else {
          this.index++
          this.host = this.hostOf(this.index)
        }
      } else {
        return this.finish()
      }
    }

    return null
  }

  /** Generates the final single `Tree` from the list of already completed trees. */
  private finish() {
    let tree: Tree = Tree.empty

    if (this.trees.length > 1) {
      const lastIndex = this.trees.length
      const positions = this.ranges.map(({ from }) => from)
      const from = positions[0]
      const to = positions[lastIndex] + this.trees[lastIndex].length
      const length = to - from
      tree = new Tree(this.language.top!, this.trees, positions, length)
    } else {
      tree = this.trees[0]
    }

    return tree
  }

  /** The current position of the parser(s). */
  get parsedPos() {
    return this.host.pos
  }

  /**
   * The position the parser will be stopping at early, if given a location
   * to stop at.
   */
  stoppedAt: number | null = null

  /**
   * Tells the parser(s) to stop early at the specified position.
   *
   * @param pos - The position to stop at.
   */
  stopAt(pos: number) {
    this.stoppedAt = pos
    const range = this.ranges[this.index]
    if (range.from <= pos && range.to >= pos) {
      this.host.stopAt(pos)
    }
  }
}
