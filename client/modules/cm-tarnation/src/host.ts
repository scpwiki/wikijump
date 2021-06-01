import { EditorParseContext } from "@codemirror/language"
import { Input, PartialParse, Tree } from "lezer-tree"
import { isEmpty, perfy } from "wj-util"
import type { TarnationLanguage } from "./language"
import { Parser, ParserBuffer, ParserContext, ParserStack } from "./parser"
import { Tokenizer, TokenizerBuffer, TokenizerContext, TokenizerStack } from "./tokenizer"
import type { ParseRegion } from "./types"

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
  private declare region: ParseRegion
  private declare context?: EditorParseContext
  private declare viewport?: { from: number; to: number }

  private declare tokenizer: Tokenizer
  private declare tokenizerPreviousRight?: TokenizerBuffer

  private declare parser: Parser

  private declare measurePerformance?: (msg?: string) => number
  declare performance?: number

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
        if (f.from > start || f.to < start) continue
        const buffer = this.language.cache.find(f.tree, start, f.to)
        if (buffer) {
          const { chunk, index } = buffer.search(this.region.from, -1)
          if (chunk && index !== null) {
            const { left, right } = buffer.split(index)
            this.tokenizerPreviousRight = right
            this.region.from = chunk.context.pos
            this.setupTokenizer(left, chunk.context)
            // check if parser is cached as well
            if (this.language.cache.has(chunk)) {
              const context = ParserContext.deserialize(this.language.cache.get(chunk)!)
              this.setupParser(context)
            }
          }
        }
      }
    }

    if (!this.tokenizer) this.setupTokenizer()
    if (!this.parser) this.setupParser()
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

  get pos() {
    return this.tokenizer.context.pos
  }

  advance(): Tree | null {
    if (!this.measurePerformance) this.measurePerformance = perfy()
    switch (this.stage) {
      case Stage.Tokenize: {
        const chunks = this.tokenizer.advance()
        if (chunks) {
          this.parser.pending = chunks
          this.stage = Stage.Parse
        } else if (
          this.tokenizerPreviousRight &&
          this.tokenizer.tryToReuse(this.tokenizerPreviousRight) &&
          this.tokenizer.done
        ) {
          this.parser.pending = this.tokenizer.chunks
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
