import { Writable, writable } from "svelte/store"
import {
  ContentFacet,
  Gutters,
  IndentHack,
  Spellcheck,
  textBuffer,
  textValue
} from "wj-codemirror"
import {
  autocompletion,
  bracketMatching,
  closeBrackets,
  drawSelection,
  EditorState,
  EditorView,
  Extension,
  highlightActiveLine,
  highlightSelectionMatches,
  highlightSpecialChars,
  history,
  indentOnInput,
  rectangularSelection,
  ViewPlugin,
  ViewUpdate
} from "wj-codemirror/cm"
import { createSheafBinding, SheafBindings } from "./extensions/bindings"
import { getSheafKeymap } from "./extensions/keymap"
import { confinement } from "./extensions/theme"
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
          highlightSpecialChars(),
          history(),
          drawSelection(),
          EditorState.allowMultipleSelections.of(true),
          indentOnInput(),
          bracketMatching(),
          closeBrackets(),
          highlightSelectionMatches(),
          autocompletion(),
          rectangularSelection(),
          highlightActiveLine(),
          EditorView.lineWrapping,
          getSheafKeymap(),
          IndentHack,
          ContentFacet.of((state, buffer) =>
            buffer ? textBuffer(state.doc) : textValue(state.doc)
          ),
          Gutters,
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
