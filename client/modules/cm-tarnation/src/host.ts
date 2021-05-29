import type { EditorParseContext } from "@codemirror/language"
import { Input, PartialParse, Tree } from "lezer-tree"
import { isEmpty, perfy } from "wj-util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserBuffer, ParserContext, ParserStack } from "./parser/index"
import {
  Tokenizer,
  TokenizerBuffer,
  TokenizerContext,
  TokenizerStack
} from "./tokenizer/index" // TODO: fix
import type { EditRegion } from "./types"

enum Stage {
  Tokenize,
  Parse
}

export class Host implements PartialParse {
  private declare language: TarnationLanguage
  private declare input: Input
  private declare stage: Stage
  private declare caching: boolean
  private declare region: EditRegion
  private declare context?: EditorParseContext
  private declare viewport?: { from: number; to: number }

  private declare tokenizer: Tokenizer
  private declare parser: Parser

  private declare measurePerformance: () => number
  declare renderPerformance?: number

  constructor(
    language: TarnationLanguage,
    input: Input,
    start: number,
    context?: EditorParseContext
  ) {
    if (isEmpty(context)) context = undefined

    this.language = language
    this.input = input
    this.stage = Stage.Tokenize
    this.caching = Boolean(context?.state)
    this.context = context

    // this.measurePerformance = perfy()
    this.measurePerformance = perfy("tarnation", 2.5)

    const reuse = false

    // get edited region
    if (reuse && context?.fragments?.length) {
      const fragments = context.fragments
      const firstFragment = fragments[0]
      const lastFragment = fragments[fragments.length - 1]

      this.region = {
        from: firstFragment.to,
        to: fragments.length === 1 ? input.length : lastFragment.from,
        offset: lastFragment.offset
      }
    } else {
      this.region = {
        from: start,
        to: input.length,
        offset: 0
      }
    }

    if (context?.viewport) this.viewport = context.viewport

    this.setupTokenizer()
    this.setupParser()
  }

  private setupTokenizer() {
    const stack = new TokenizerStack({ stack: [["root", {}]], embedded: null })
    const context = new TokenizerContext(this.region.from, stack)
    const buffer = new TokenizerBuffer()

    this.tokenizer = new Tokenizer(
      this.language,
      context,
      buffer,
      this.input,
      this.region
    )
  }

  private setupParser() {
    const buffer = new ParserBuffer()
    const stack = new ParserStack()
    const context = new ParserContext(this.region.from, 0, buffer, stack, {
      pending: [],
      parsers: []
    })

    this.parser = new Parser(this.language, context, this.input)
  }

  get pos() {
    return this.tokenizer.context.pos
  }

  advance(): Tree | null {
    switch (this.stage) {
      case Stage.Tokenize: {
        const tokens = this.tokenizer.advance()
        if (tokens) {
          this.parser.pending = tokens
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

  private finish(buffer: number[], reused: Tree[]): Tree {
    const length = this.pos - this.region.from

    const tree = Tree.build({
      topID: 0,
      nodeSet: this.language.nodes!.set,
      buffer,
      reused,
      length,
      start: this.region.from
    })

    this.renderPerformance = this.measurePerformance()

    return tree
  }

  forceFinish(): Tree {
    // TODO: make this actually stop where it is and not just advance fully
    if (this.stage === Stage.Tokenize) {
      const tokens = this.tokenizer.advanceFully()
      this.parser.pending = tokens
      this.stage = Stage.Parse
    }

    const { buffer, reused } = this.parser.advanceFully()
    return this.finish(buffer, reused)
  }
}
