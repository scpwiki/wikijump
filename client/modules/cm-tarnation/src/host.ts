import { EditorParseContext } from "@codemirror/language"
import { Input, PartialParse, Tree } from "lezer-tree"
import { isEmpty, perfy } from "wj-util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserBuffer, ParserContext, ParserStack } from "./parser"
import { Tokenizer, TokenizerBuffer, TokenizerContext, TokenizerStack } from "./tokenizer"
import type { EditRegion } from "./types"

enum Stage {
  Tokenize,
  Parse
}

export class Host implements PartialParse {
  private declare language: TarnationLanguage
  private declare input: Input
  private declare start: number
  private declare stage: Stage
  private declare caching: boolean
  private declare region: EditRegion
  private declare context?: EditorParseContext
  private declare viewport?: { from: number; to: number }

  private declare tokenizer: Tokenizer
  private declare parser: Parser

  private declare measurePerformance: (msg?: string) => number
  declare renderPerformance?: number

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
    this.caching = Boolean(context?.state)
    this.context = context

    // this.measurePerformance = perfy()
    this.measurePerformance = perfy("tarnation", 2.5)

    // get edited region
    if (context?.fragments?.length) {
      const fragments = context.fragments
      const firstFragment = fragments[0]
      const lastFragment = fragments[fragments.length - 1]
      const onlyOne = fragments.length === 1

      if (onlyOne) {
        this.region = {
          from: start,
          // to: firstFragment.from,
          to: input.length,
          offset: firstFragment.offset
        }
      } else {
        this.region = {
          from: Math.max(firstFragment.to, start),
          // to: lastFragment.from,
          to: input.length,
          offset: lastFragment.offset
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
        offset: 0
      }
    }

    // find cached data, if possible
    if (context?.fragments?.length) {
      for (let idx = 0; idx < context.fragments.length; idx++) {
        const f = context.fragments[idx]
        if (f.from > start || f.to < start) continue
        const bundle = this.language.cache.find(f.tree, start, f.to)
        if (bundle) {
          const { chunk, index } = bundle.tokenizerBuffer.search(this.region.from, -1)
          if (chunk && index !== null) {
            const { left, right } = bundle.tokenizerBuffer.split(index)
            const tokenizerContext = chunk.context
            const tokenizerBuffer = left
            this.region.from = tokenizerContext.pos
            this.setupTokenizer(tokenizerBuffer, tokenizerContext)
          }
        }
      }
    }

    if (!this.tokenizer) this.setupTokenizer()
    this.setupParser()
  }

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

  private setupParser() {
    const buffer = new ParserBuffer()
    const stack = new ParserStack()
    const context = new ParserContext(this.start, 0, buffer, stack, {
      pending: [],
      parsers: []
    })

    this.parser = new Parser(this.language, context, this.input, [], this.context)
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

  private finish(buffer: number[], reused: Tree[], forced = false): Tree {
    const length = this.pos - this.start

    const tree = Tree.build({
      topID: 0,
      nodeSet: this.language.nodes!.set,
      buffer,
      reused,
      length,
      start: this.start
    })

    this.language.cache.attach(this.tokenizer.buffer, this.parser.context, tree)

    if (this.context?.skipUntilInView && length < this.input.length) {
      this.context.skipUntilInView(this.pos, this.input.length)
    }

    this.renderPerformance = this.measurePerformance(forced ? "forced" : "")

    return tree
  }

  forceFinish(): Tree {
    switch (this.stage) {
      case Stage.Tokenize: {
        const tokens = this.tokenizer.compile()
        const { buffer, reused } = this.parser.forceTokens(tokens)
        return this.finish(buffer, reused, true)
      }
      case Stage.Parse: {
        // TODO: determine approximate document position to advance only as far as needed
        const { buffer, reused } = this.parser.forceFinish()
        return this.finish(buffer, reused, true)
      }
    }
  }
}
