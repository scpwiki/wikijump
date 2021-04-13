import { createContext, GrammarToken } from './grammar/grammar'
import { klona } from 'klona'
import type * as DF from './grammar/definition'
import type { NodeMap, State } from './index'
import type { Context } from './buffer'
import type { Input } from 'lezer-tree'

/** Directs the parser to nest tokens using the node's type ID. */
export type MappedParserAction = [id: number, inclusive: number][]

/** A more efficient representation of `GrammarToken` used in the parser.  */
export type MappedToken =
  [type: number, from: number, to: number, open?: MappedParserAction, close?: MappedParserAction]

/** Represents a region of an embedded language. */
export type EmbeddedRange = { lang: string, start: number, end: number }

/** Represents a stack node in a {@link TokenizerStack}. */
export type TokenizerStackElement = [state: string, context: DF.Context]
/** A serialized (just data) form of a {@link TokenizerStack}. */
export interface SerializedTokenizerStack {
  stack: TokenizerStackElement[]
  embedded: null | { lang: string, start: number }
}

/** Handles the tokenization of a {@link Grammar}. */
export class Tokenizer {

  declare stack?: TokenizerStack

  declare private lastInput?: Input
  declare private lastStr?: string
  declare private lastPos?: number

  /** Determines how wide of an input region will be tokenized. */
  private padding = 1000

  constructor(private state: State, context?: Context) {
    if (context) this.context = context
  }

  get grammar() { return this.state.grammar }
  get nodes() { return this.state.nodes }

  set context(context: Context) {
    this.stack = context.tokenizer
    if (this.stack.depth === 0) {
      this.stack.push(this.grammar.start ?? 'root')
    }
  }

  /** Compiles a {@link GrammarToken} into a {@link MappedToken}.
   *  This primarily involves remapping the token names into IDs. */
  private static compileToken(token: GrammarToken, nodes: NodeMap): MappedToken {
    const { type, from, to, open, close } = token
    const out: MappedToken = [nodes.get(type)!, from, to]
    if (open) out[3] = open.map(([ type, inclusive ]) => [nodes.get(type)!, inclusive])
    if (close) out[4] = close.map(([ type, inclusive ]) => [nodes.get(type)!, inclusive])
    return out
  }

  /** Returns whether or not the given last {@link MappedToken} can be merged
   *  with the next given {@link GrammarToken}. */
  private canContinue(last?: MappedToken, next?: GrammarToken) {
    if (!last || !next) return false                             // tokens are invalid
    if (last.length > 2 || next.open || next.close) return false // parser directives present
    if (last[0] === -1 || next.embedded) return false            // embedded handling token
    if (last[0] !== this.nodes.get(next.type)) return false      // types aren't equivalent
    if (last[2] !== next.from) return false                      // tokens aren't inline
    return true
  }

  /** Executes a tokenization step. */
  exec(input: Input, pos: number) {
    if (!this.stack) throw new Error('Attempted to tokenize without a stack/context being set!')
    const { grammar, nodes, stack, padding, lastInput, lastPos, lastStr } = this

    let str: string
    let start: number

    // prevent a document slice by reusing the last string made if it seems safe
    if (lastStr && lastPos && lastInput === input && pos > lastPos && (pos - lastPos < padding / 2)) {
      str = lastStr
      start = (lastPos < padding ? lastPos : padding) + (pos - lastPos)
    } else {
      str = input.read(pos - padding, pos + padding)
      start = pos < padding ? pos : padding
      this.lastInput = input
      this.lastStr = str
      this.lastPos = pos
    }

    const match = grammar.match(createContext(stack.state, stack.context), str, start, pos)

    if (!match) return { tokens: null, popped: null, length: 1 }

    const tokens = match.compile()

    if (!tokens.length) return { tokens: null, popped: null, length: match.length }

    const mapped = new Set<MappedToken>()
    const popped = new Set<EmbeddedRange>()

    let last!: MappedToken

    for (const token of tokens) {
      const { next, switchTo, embedded, context, from, to } = token

      stack.changed = false
      let pushEmbedded = false

      if (embedded) {
        if (!stack.embedded && embedded.endsWith('!')) {
          mapped.add(last = [-1, from, to])
          popped.add({ lang: embedded.slice(0, embedded.length - 1), start: from, end: to })
          continue
        } else if (embedded === '@pop') {
          popped.add(stack.endEmbedded(from))
        } else if (!stack.embedded) {
          pushEmbedded = true
          stack.setEmbedded(embedded, to)
        }
      }

      if (next) switch (next) {
        case '@pop': stack.pop(); break
        case '@popall': stack.popall(); break
        case '@push': stack.push(next, context); break
        default: stack.push(next, context); break
      } else if (switchTo) {
        stack.switchTo(switchTo, context)
      } else if (context) {
        stack.context = context
      }

      if (!token.empty && (!stack.embedded || pushEmbedded)) {
        if (last && !stack.changed && this.canContinue(last, token)) last[2] = token.to
        else mapped.add(last = Tokenizer.compileToken(token, nodes))
      }

      if (pushEmbedded) mapped.add(last = [-1, to, to])
    }

    return { tokens: mapped, popped, length: match.length }
  }
}

/** State/stack object for a {@link Tokenizer}. */
export class TokenizerStack {

  /** Specifies if the state has changed since the last time this property has been set. */
  changed = false

  /** Embedded language data, if present. */
  embedded: null | { lang: string, start: number }

  /** The internal stack. */
  declare private stack: TokenizerStackElement[]

  constructor(serialized: SerializedTokenizerStack = { stack: [], embedded: null }) {
    const { stack, embedded } = klona(serialized)
    this.stack = stack
    this.embedded = embedded
  }

  private get last() { return this.stack[this.stack.length - 1]  }
  private set last(element) { this.stack[this.stack.length - 1] = element }

  /** The top-most state of the stack. */
  get state() { return this.last[0] }
  /** The length (depth), or number of stack nodes, in the stack. */
  get depth() { return this.stack.length }
  /** The parent of the stack, i.e. the state of the stack if it were to be popped. */
  get parent() { const parent = this.clone(); parent.pop(); return parent }

  get context() { return this.last?.[1] ?? {} }
  set context(context) { this.last[1] = context ?? {} }

  /** Push a new state to the top of the stack. */
  push(state: string, context = this.context) {
    this.changed = true
    this.stack.push([state, context])
  }

  /** Switch to a new state, replacing the current one. */
  switchTo(state: string, context = this.context) {
    this.changed = true
    this.last = [state, context]
  }

  /** Remove the top-most state of the stack. */
  pop() {
    this.changed = true
    return this.stack.pop()?.[0]
  }

  /** Remove all states from the stack except the very first. */
  popall() {
    this.changed = true
    this.stack = [this.stack.shift() ?? ['root', {}]]
  }

  /** Returns a deep clone of the stack. */
  clone() {
    return new TokenizerStack(this.serialize())
  }

  /** Sets the embedded data. */
  setEmbedded(lang: string, start: number) {
    this.changed = true
    this.embedded = { lang, start }
  }

  /** Removes the embedded data. */
  endEmbedded(end: number): EmbeddedRange {
    if (!this.embedded) throw new Error('Tried to end a non-existent embedded range!')
    this.changed = true
    const embedded = this.embedded
    this.embedded = null
    return { ...embedded, end }
  }

  /** Serializes the stack and embedded data into a list of strings. */
  serialize(): SerializedTokenizerStack {
    const { stack, embedded } = this
    return { stack: klona(stack), embedded: klona(embedded) }
  }

  /** Compares two stacks and returns if they are equal.
  *  They can be pure `MonarchStack` objects or already serialized. */
  static isEqual(stack1: SerializedTokenizerStack | TokenizerStack, stack2: SerializedTokenizerStack | TokenizerStack) {
    // convert to just string arrays
    if ('serialize' in stack1) stack1 = stack1.serialize()
    if ('serialize' in stack2) stack2 = stack2.serialize()
    // check lengths
    if (stack1.stack.length !== stack2.stack.length) return false
    // check for every value
    return stack1.stack.every(([str], idx) => str === (stack2 as SerializedTokenizerStack).stack[idx][0])
  }
}
