import { getActiveLines, Gutters, textBuffer, textValue } from "wj-codemirror"
import { EditorState, EditorView, syntaxTree, Text } from "wj-codemirror/cm"
import type { SheafCore } from "./core"
import type { SheafBindings } from "./extensions/bindings"

export interface SheafStateConstructorOpts {
  self: SheafCore
  view: EditorView
  bindings: SheafBindings
}

export class SheafState {
  declare readonly self: SheafCore
  declare readonly parent: Element
  declare readonly view: EditorView
  declare readonly bindings: SheafBindings
  declare readonly state: EditorState
  declare readonly doc: Text
  declare readonly activeLines: Set<number>

  constructor(opts: SheafStateConstructorOpts) {
    this.self = opts.self
    this.view = opts.view
    this.bindings = opts.bindings
    this.state = opts.view.state
    this.doc = opts.view.state.doc
    this.activeLines = getActiveLines(opts.view.state)
  }

  private declare _value?: string

  async value() {
    if (this._value) return this._value
    return (this._value = await textValue(this.doc))
  }

  async buffer() {
    return await textBuffer(this.doc)
  }

  get tree() {
    return syntaxTree(this.state)
  }

  get gutters() {
    return Gutters.get(this.state)
  }

  set gutters(state: boolean) {
    Gutters.set(this.view, state)
  }

  extend(opts?: Partial<SheafStateConstructorOpts>) {
    return new SheafState({
      self: this.self,
      view: this.view,
      bindings: this.bindings,
      ...opts
    })
  }
}
