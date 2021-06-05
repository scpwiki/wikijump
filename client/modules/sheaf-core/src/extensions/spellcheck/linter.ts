/* eslint-disable @typescript-eslint/unbound-method */
import { EditorView, ViewPlugin, ViewUpdate } from "@codemirror/view"
import { ContentFacet } from "../content"
import { Spellcheck } from "./extension"
import { Spellchecker } from "./spellchecker"

/**
 * `ViewPlugin` that spins-up an automatic scanner for spellchecking. Acts
 * much like a normal CodeMirror linter.
 */
export const spellcheckLinter = ViewPlugin.fromClass(
  class {
    declare view: EditorView
    declare timeout // untyped because TS types setTimeout badly

    constructor(view: EditorView) {
      this.view = view
      // ensures that "this" gets preserved regardless of how the function is ran
      this.run = this.run.bind(this)
      this.timeout = setTimeout(this.run, 250)
    }

    update(update: ViewUpdate) {
      if (update.docChanged) {
        clearTimeout(this.timeout)
        this.timeout = setTimeout(this.run, 250)
      }
    }

    /**
     * Runs a spellcheck on the document, and updates the editor's state.
     * This function is potentially extremely long-running and thus
     * automatically cancels if the document changes.
     */
    async run() {
      const state = this.view.state
      const extract = state.facet(ContentFacet)
      const content = await extract(state, true)
      const misspellings = await Spellchecker.checkWords(content)

      // check if the document changed while we were processing
      if (this.view.state.doc !== state.doc) return

      const field = Spellcheck.get(this.view)
      if (field.enabled) {
        Spellcheck.set(this.view, field.set(misspellings ?? []))
      }
    }
  }
)
