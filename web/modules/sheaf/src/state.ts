import { syntaxTree } from "@codemirror/language"
import type { EditorState, Text } from "@codemirror/state"
import type { EditorView } from "@codemirror/view"
import type { Tree } from "@lezer/common"
import {
  getActiveLines,
  Gutters,
  printTree,
  textBuffer,
  textValue
} from "@wikijump/codemirror"
import FTML, {
  type SyntaxTree,
  type Token,
  type Warning
} from "@wikijump/ftml-wasm-worker"
import { Memoize } from "typescript-memoize"
import type { SheafCore } from "./core"
import type { SheafBindings } from "./extensions/bindings"

export interface SheafStateConstructorOpts {
  self: SheafCore
  view: EditorView
  bindings: SheafBindings
}

export class SheafState {
  /** The {@link SheafCore} instance this state is describing. */
  declare readonly self: SheafCore

  /** The parent DOM element the editor is attached to. */
  declare readonly parent: Element

  /** The `EditorView` of the CM6 instance. */
  declare readonly view: EditorView

  /** The bindings Sheaf is using. */
  declare readonly bindings: SheafBindings

  /** The current `EditorState` of the CM6 instance. */
  declare readonly state: EditorState

  /** The `Text` object from `state.doc`. */
  declare readonly doc: Text

  /** The currently active line numbers. */
  declare readonly activeLines: Set<number>

  /** @param opts - The starting state configuration to use. */
  constructor(opts: SheafStateConstructorOpts) {
    this.self = opts.self
    this.view = opts.view
    this.bindings = opts.bindings
    this.state = opts.view.state
    this.doc = opts.view.state.doc
    this.activeLines = getActiveLines(opts.view.state)
  }

  /** Retrieves the value of the editor. */
  @Memoize()
  async value() {
    return await textValue(this.doc)
  }

  /** Retrieves the value of the editor as an `Uint8Array`. */
  @Memoize()
  async buffer() {
    return await textBuffer(this.doc)
  }

  /** Renders the editor with FTML. */
  @Memoize()
  async render() {
    return await FTML.renderHTML(await this.value(), undefined, "draft")
  }

  /** Renders the document to HTML. */
  @Memoize()
  async html(format = false) {
    const { html } = await this.render()
    return format ? await FTML.formatHTML(html) : html
  }

  /** Renders the document and returns its stylesheets. */
  @Memoize()
  async styles() {
    const { styles } = await this.render()
    return styles
  }

  /** Renders the document and returns its combined stylesheet. */
  @Memoize()
  async style() {
    const { styles } = await this.render()
    return styles
      .map((style, idx) => `/* stylesheet ${idx + 1} */\n\n${style}\n\n`)
      .join("\n")
  }

  /** Gets the document's resultant FTML AST and warnings. */
  @Memoize()
  async parse() {
    return await FTML.parse(await this.value())
  }

  /** Gets the document's resultant FTML AST. */
  @Memoize()
  async ast(): Promise<SyntaxTree> {
    const { ast } = await this.parse()
    return ast
  }

  /** Gets a pretty-printed JSON version of the FTML AST. */
  @Memoize()
  async prettyAST() {
    const ast = await this.ast()
    return JSON.stringify(ast, undefined, 2)
  }

  /** Gets the document's FTML emitted warnings. */
  @Memoize()
  async warnings(): Promise<Warning[]> {
    const { warnings } = await this.parse()
    return warnings
  }

  /** Tokenizes the document with FTML. */
  @Memoize()
  async tokenize(): Promise<Token[]> {
    return await FTML.tokenize(await this.value())
  }

  /** Tokenizes the document and returns the result as a pretty-printed string. */
  @Memoize()
  async inspectTokens() {
    return await FTML.inspectTokens(await this.value())
  }

  /** Gets the word count via inspection of the FTML AST. */
  @Memoize()
  async wordCount() {
    return await FTML.wordCount(await this.value())
  }

  /** Creates a pretty printed version of the editor's syntax tree. */
  @Memoize()
  async prettyEditorAST() {
    return printTree(this.tree, await this.value())
  }

  /** The current _editor_ syntax tree. */
  get tree(): Tree {
    return syntaxTree(this.state)
  }

  /** True if the gutters are being shown. Can be set. */
  get gutters() {
    return Gutters.get(this.state)
  }

  /** True if the gutters are being shown. Can be set. */
  set gutters(state: boolean) {
    Gutters.set(this.view, state)
  }

  /**
   * Extends this state and creates a new one.
   *
   * @param opts - Options to pass to the new state.
   */
  extend(opts?: Partial<SheafStateConstructorOpts>) {
    // TODO: reimplement memoization manually
    // so that if the documentdoesn't change, we copy over the rendered result
    return new SheafState({
      self: this.self,
      view: this.view,
      bindings: this.bindings,
      ...opts
    })
  }
}
