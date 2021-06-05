import { EditorState, Extension } from "@codemirror/state"
import { EditorView, ViewPlugin, ViewUpdate } from "@codemirror/view"
import { Writable, writable } from "svelte/store"
import { getSheafBasicExtensions } from "../extensions/base"
import { createSheafBinding, SheafBindings } from "../extensions/bindings"
import { ContentFacet } from "../extensions/content"
import { gutters } from "../extensions/gutters"
import { indentHack } from "../extensions/indent-hack"
import { getSheafKeymap } from "../extensions/keymap"
import { Spellcheck } from "../extensions/spellcheck"
import { confinement } from "../extensions/theme"
import { textBuffer, textValue } from "../util/misc"
import { SheafState } from "./state"

export class SheafCore {
  declare state: SheafState
  private declare store: Writable<SheafState>
  declare subscribe: Writable<SheafState>["subscribe"]
  declare set: Writable<SheafState>["set"]

  constructor(doc: string, bindings: SheafBindings = {}, extensions: Extension[] = []) {
    const updateHandler = ViewPlugin.define(() => ({
      update: viewUpdate => this.update(viewUpdate)
    }))

    const view = new EditorView({
      state: EditorState.create({
        doc,
        extensions: [
          getSheafBasicExtensions(),
          getSheafKeymap(),
          indentHack,
          ContentFacet.of((state, buffer) =>
            buffer ? textBuffer(state.doc) : textValue(state.doc)
          ),
          gutters,
          Spellcheck,
          confinement,
          createSheafBinding(this, bindings),
          extensions,
          updateHandler
        ]
      })
    })

    this.state = new SheafState({ self: this, view, bindings })
    this.store = writable(this.state)
    this.subscribe = this.store.subscribe
    this.set = this.store.set
  }

  private update(update: ViewUpdate) {
    if (!update.docChanged && !update.selectionSet) return
    this.state = this.state.extend()
    this.store.set(this.state)
  }

  mount(element: Element) {
    element.appendChild(this.state.view.dom)
  }

  /**
   * Destroys the editor. Usage of the editor object after destruction is
   * obviously not recommended.
   */
  destroy() {
    this.state.view.destroy()
  }
}
