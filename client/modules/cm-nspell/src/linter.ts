/* eslint-disable @typescript-eslint/unbound-method */
import { EditorView, ViewPlugin, ViewUpdate } from "wj-codemirror/cm"
import { Spellcheck } from "./extension"
import nspell from "./nspell"
import { viewWords } from "./tokenizer"

class SpellcheckLinter {
  declare id: number
  declare view: EditorView
  declare timeout // untyped because TS types setTimeout badly

  constructor(view: EditorView) {
    this.view = view
    // ensures that "this" gets preserved regardless of how the function is ran
    this.run = this.run.bind(this)
    this.id = Math.random()
    this.timeout = setTimeout(this.run, 250)
  }

  update(update: ViewUpdate) {
    if (update.docChanged || update.viewportChanged) {
      this.id = Math.random()
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
    const id = this.id

    const words = viewWords(this.view)
    const misspelled = await nspell.misspelled(words)

    if (id !== this.id) return

    const field = Spellcheck.get(this.view)
    if (field.enabled) {
      Spellcheck.set(this.view, field.set(misspelled))
    }
  }
}

/**
 * `ViewPlugin` that spins-up an automatic scanner for spellchecking. Acts
 * much like a normal CodeMirror linter.
 */
export const spellcheckLinterPlugin = ViewPlugin.fromClass(SpellcheckLinter)
