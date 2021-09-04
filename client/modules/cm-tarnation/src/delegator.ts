import {
  Input,
  parseMixed,
  Parser,
  PartialParse,
  Tree,
  TreeCursor,
  TreeFragment
} from "@lezer/common"
import { LanguageDescription, ParseContext } from "modules/wj-codemirror/cm"
import { EmbeddedParserProp, EmbeddedParserType } from "."
import { Host } from "./host"
import { TarnationLanguage } from "./language"

export class DelegatorFactory extends Parser {
  constructor(private language: TarnationLanguage) {
    super()
  }

  createParse(
    input: Input,
    fragments: TreeFragment[],
    ranges: { from: number; to: number }[]
  ) {
    const delegator = new Delegator(this.language, input, fragments, ranges)
    const wrapper = parseMixed(this.nest.bind(this))
    const wrapped = wrapper(delegator, input, fragments, ranges)
    return wrapped
  }

  private nest(node: TreeCursor, input: Input) {
    if (node.type === EmbeddedParserType && node.tree) {
      // get name from the per-node property
      const name = node.tree.prop(EmbeddedParserProp)
      if (!name) return null

      const lang = LanguageDescription.matchLanguageName(
        this.language.nestLanguages,
        name
      )
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

export class Delegator implements PartialParse {
  private declare index: number

  private declare host: Host

  private declare trees: Tree[]

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

  private hostOf(index: number) {
    return new Host(
      this.language,
      this.input,
      this.fragments,
      this.ranges[index],
      this.ranges.length <= 1
    )
  }

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

  get parsedPos() {
    return this.host.pos
  }

  stoppedAt: number | null = null

  stopAt(pos: number) {
    this.stoppedAt = pos
    const range = this.ranges[this.index]
    if (range.from <= pos && range.to >= pos) {
      this.host.stopAt(pos)
    }
  }
}
