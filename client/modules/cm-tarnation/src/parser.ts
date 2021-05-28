import { EditorParseContext, LanguageDescription } from "@codemirror/language"
import { klona } from "klona"
import { Input, PartialParse, Tree } from "lezer-tree"
import { perfy } from "wj-util"
import { Buffer, BufferCache, BufferToken, Context } from "./buffer"
import type { State } from "./index"
import type { EmbeddedRange, Tokenizer } from "./tokenizer"

export type ParserElementStack = [name: number, start: number, children: number][]

export interface SerializedEmbedded {
  pending: BufferToken[]
  parsers: [token: BufferToken, range: EmbeddedRange][]
}

export class Parser {
  private declare caching: boolean
  private declare context: Context
  private declare embed: EmbeddedHandler
  private declare tokenizer: Tokenizer
  private declare viewport?: { from: number; to: number }

  private buffer = new Buffer()

  /** Number of parse steps per `advance()` call. */
  private steps = 10
  private lookaheadMargin = 50 // chars
  private checkpointSpacing = 250 // chars
  private logPerf?: () => void
  private debug = false

  private lastCheckpointPos = 0

  pos = 0

  constructor(
    private state: State,
    private input: Input,
    private start: number,
    private editorContext?: EditorParseContext
  ) {
    // check if editor context is just an empty object
    // it should always have fragments if it's real
    if (editorContext && !editorContext.fragments) {
      editorContext = undefined
      this.editorContext = undefined
    }

    this.caching = Boolean(editorContext?.state)

    if (this.caching) {
      this.viewport = editorContext!.viewport
      if (editorContext?.fragments?.length) {
        for (const f of editorContext.fragments) {
          if (f.from > start || f.to < start) continue
          const buffer = Parser.findBuffer(state.cache, f.tree, start, f.to)
          if (buffer) {
            const found = buffer.findContext(start, f.to - this.lookaheadMargin)
            if (found) {
              const { context, index } = found
              this.lastCheckpointPos = context.pos
              this.buffer = buffer.cut(index, true)
              this.context = context
              this.pos = context.pos
              // if (this.debug) console.log(`starting at: ${editorContext.state.doc.lineAt(this.pos).number}`)
              break
            }
          }
        }
      }
    }

    if (!this.context) {
      this.context = new Context(this.start)
      this.pos = this.start
    }

    this.embed = new EmbeddedHandler(state, input, this.context, editorContext)
    this.tokenizer = state.tokenizer
    this.tokenizer.context = this.context

    if (this.debug) this.logPerf = perfy("parser", 2.5)
  }

  private get doc() {
    return this.editorContext?.state.doc ?? null
  }

  private get ended() {
    return this.pos >= this.input.length
  }

  private skipped = false
  private get skipping() {
    if (this.skipped) return true
    if (this.ended || !this.editorContext || !this.embed.done) return false

    const { viewport, start, pos } = this

    // viewport diff hopefully adds enough buffer
    if (start <= viewport!.to && pos >= viewport!.to + (viewport!.to - viewport!.from)) {
      this.skipped = true
      return true
    }

    return false
  }

  private static findBuffer(
    cache: BufferCache,
    tree: Tree,
    startPos: number,
    before: number,
    off = 0
  ): Buffer | null {
    const buffer =
      off >= startPos && off + tree.length >= before ? cache.get(tree) : undefined
    if (buffer) return buffer
    // check children
    for (let i = tree.children.length - 1; i >= 0; i--) {
      const child = tree.children[i]
      const pos = off + tree.positions[i]
      if (!(child instanceof Tree && pos < before)) continue
      const found = this.findBuffer(cache, child, startPos, before, pos)
      if (found) return found
    }
    return null
  }

  private compileTree() {
    const { start, buffer, state, doc } = this

    let length = this.pos - start

    // filthy
    // not sure why this is needed, to be honest
    // the length of the document can somehow be disjointed from the input length
    // this causes issues when remapping the tree, apparently
    if (doc && doc.length < length) length = doc.length

    const tree = state.buildTree(buffer, start, length)

    return tree
  }

  private finish() {
    const { input, buffer, context, pos, skipping, editorContext, viewport } = this
    const { parser: stack } = context

    // handle unfinished stack
    while (stack.length) {
      const [startid, startpos, children] = stack.pop()!
      buffer.add([startid, startpos, pos, children * 4 + 4])
      stack.increment()
    }

    if (skipping) editorContext!.skipUntilInView(viewport!.to, input.length)

    const tree = this.compileTree()
    if (this.debug && this.logPerf) this.logPerf()
    // if (this.debug && editorContext) {
    //   console.log(`ending at: ${editorContext.state.doc.lineAt(this.pos).number}`)
    // }
    return tree
  }

  private step() {
    if (!this.ended && !this.skipped) {
      const { input, buffer, context, embed, tokenizer } = this
      const { parser: stack } = context

      let pos = this.pos

      const { tokens, popped, length } = tokenizer.exec(input, pos) ?? {}

      if (tokens) {
        for (const token of tokens) {
          const [type, from, to, open, close] = token
          if (type === -1) {
            const token = buffer.add([0, from, to, -1, Tree.empty]) as BufferToken
            stack.increment()
            embed.push(token)
            continue
          }

          // opening
          if (open) {
            open.forEach(([id, inclusive]) => {
              stack.push(id, inclusive ? from : to, type ? (inclusive ? 0 : -1) : 0)
            })
          }

          // closing
          let pushed = false
          if (close && stack.length) {
            close.forEach(([id, inclusive]) => {
              const idx = stack.last(id)
              if (idx !== null) {
                // cuts off anything past our closing stack element
                stack.close(idx)
                // if we're inclusive of the end token we need to push the token early
                if (type && inclusive && !pushed) {
                  buffer.add([type, from, to, 4])
                  stack.increment()
                  pushed = true
                }
                const [startid, startpos, children] = stack.pop()!
                buffer.add([startid, startpos, inclusive ? to : from, children * 4 + 4])
                stack.increment()
              }
            })
          }

          // token itself
          if (type && !pushed) {
            buffer.add([type, from, to, 4])
            stack.increment()
          }
        }
      }

      if (popped) for (const range of popped) embed.push(range)

      pos += length

      const ended = this.ended

      if (ended) pos = input.length
      this.pos = pos
      context.pos = pos
      context.embed = embed.serialize()

      if (
        this.caching &&
        !ended &&
        pos - this.lastCheckpointPos >= this.checkpointSpacing
      ) {
        this.lastCheckpointPos = pos
        buffer.add(context)
      }
    }

    // advance nested
    this.embed.advance()
  }

  forceFinish() {
    const { input, pos } = this

    // if we're very close to the end, we'll finish the parse anyways
    // this catches some flukey behavior
    if (input.length - pos < 500) {
      let done: Tree | null = null
      while (!(done = this.advance())) {}
      return done
    }

    return this.compileTree()
  }

  advance() {
    for (let step = this.steps; step > 0; step--) this.step()

    if ((this.ended || this.skipping) && this.embed.done) return this.finish()
    return null
  }
}

export class ParserStack {
  declare stack: ParserElementStack

  constructor(stack: ParserElementStack = []) {
    this.stack = stack.map(element => [...element])
  }

  get length() {
    return this.stack.length
  }

  /** Add a child to every element. */
  increment() {
    this.stack.forEach(element => element[2]++)
  }

  /** Add a new element. */
  push(id: number, start: number, children: number) {
    this.stack.push([id, start, children])
  }

  /** Remove and return the last element. */
  pop() {
    return this.stack.pop()
  }

  /** Remove every element past the index given. */
  close(idx: number) {
    this.stack = this.stack.slice(0, idx + 1)
  }

  /** Returns the last element with the given ID. */
  last(id: number) {
    const idx = this.stack.map(element => element[0]).lastIndexOf(id)
    if (idx === -1) return null
    return idx
  }

  /** Returns a safe copy of the stack's internal array. */
  serialize() {
    return klona(this.stack)
  }
}

export class EmbeddedHandler {
  private declare start: number

  private pending: BufferToken[] = []
  private parsers: {
    token: BufferToken
    lang: EmbeddedLanguage
    parser?: PartialParse
  }[] = []

  fullyLoaded = true

  constructor(
    private state: State,
    private input: Input,
    private context: Context,
    private editorContext?: EditorParseContext
  ) {
    this.start = this.context.pos
    const embed = this.context.embed
    this.pending = [...embed.pending]
    this.parsers = embed.parsers.map(([token, range]) => ({
      lang: new EmbeddedLanguage(this.state, range),
      token
    }))
  }

  get done() {
    return this.parsers.length === 0
  }

  push(embed: BufferToken | EmbeddedRange) {
    if (embed instanceof BufferToken) this.pending.push(embed)
    else {
      if (this.pending.length === 0) {
        throw new Error("Attempted to push an unassigned language!")
      }
      const token = this.pending.shift()!
      const lang = new EmbeddedLanguage(this.state, embed)
      this.parsers.push({ token, lang })
    }
  }

  advance() {
    if (this.done) return true
    const { parsers, start: startPos, editorContext } = this
    const { token, lang } = parsers[0]
    const { start, end } = lang.range

    if (token.tree !== Tree.empty && startPos >= end) {
      parsers.shift()
      if (this.done) return true
      return null
    }

    const parser = (parsers[0].parser ||= lang.parse(
      this.input.clip(end),
      start,
      editorContext
    ))

    // effectively marks the tree as stale
    if (token.tree !== Tree.empty) token.tree = Tree.empty

    const done = parser.advance()
    if (done) {
      token.tree = done
      parsers.shift()
      if (!lang.ready) this.fullyLoaded = false
      if (this.done) return true
    }
    return null
  }

  serialize(): SerializedEmbedded {
    return {
      pending: [...this.pending],
      parsers: this.parsers.map(parser => [parser.token, klona(parser.lang.range)])
    }
  }
}

/**
 * Fake {@link PartialParse} implementation that immediately returns a
 * specified {@link Tree}.
 */
class FakeParse {
  constructor(private input: Input, private tree: Tree) {}
  get pos() {
    return this.input.length
  }
  advance() {
    return this.tree
  }
  forceFinish() {
    return this.tree
  }
}

class EmbeddedLanguage {
  private declare loading
  private declare parser

  lang: LanguageDescription | null = null

  constructor(public state: State, public range: EmbeddedRange) {
    if (state.nestLanguages.length) {
      this.lang = LanguageDescription.matchLanguageName(state.nestLanguages, range.lang)
      if (this.lang?.support) this.parser = this.bindParser()
      else this.loading = this.init(this.lang)
    }
  }

  get ready() {
    return Boolean(this.parser)
  }

  private bindParser() {
    if (!this.lang?.support) throw new Error("Could not bind unloaded language!")
    const parser = this.lang.support.language.parser
    return parser.startParse.bind(parser)
  }

  private async init(lang: LanguageDescription | null) {
    if (!lang) this.parser = input => new FakeParse(input, Tree.empty)
    else {
      await lang.load()
      return (this.parser = this.bindParser())
    }
  }

  private fallbackParser(context?: EditorParseContext) {
    return !context
      ? (input: Input) => new FakeParse(input, Tree.empty)
      : // eslint-disable-next-line @typescript-eslint/unbound-method
        EditorParseContext.getSkippingParser(this.loading).startParse
  }

  parse(input: Input, start: number, context?: EditorParseContext) {
    return (this.parser ?? this.fallbackParser(context))(input, start, context ?? {})
  }
}
