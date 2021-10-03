/* eslint-disable @typescript-eslint/unbound-method */
import { EditorView, ViewPlugin, ViewUpdate } from "@wikijump/codemirror/cm"
import { timeout, Timeout } from "@wikijump/util"
import espells from "./espells"
import { Spellcheck } from "./extension"
import { getLocale } from "./locales"
import { visibleWords } from "./tokenizer"

class SpellcheckLinter {
  declare id: number
  declare view: EditorView
  declare timeout: Timeout<Promise<void>>

  constructor(view: EditorView) {
    this.view = view
    // ensures that "this" gets preserved regardless of how the function is ran
    this.run = this.run.bind(this)
    this.id = Math.random()
    this.timeout = timeout(250, this.run)
  }

  update(update: ViewUpdate) {
    if (update.docChanged || update.viewportChanged) {
      this.id = Math.random()
      this.timeout.reset()
    }
  }

  /**
   * Runs a spellcheck on the document, and updates the editor's state.
   * This function is potentially extremely long-running and thus
   * automatically cancels if the document changes.
   */
  async run() {
    const id = this.id

    const words = visibleWords(this.view, getLocale(espells.locale))
    const flagged = await espells.check(words)

    if (id !== this.id) return

    const field = Spellcheck.get(this.view)
    if (field.enabled) {
      Spellcheck.set(this.view, field.set(flagged))
    }
  }
}

/** `ViewPlugin` that spins-up an automatic scanner for spellchecking. */
export const spellcheckLinterPlugin = ViewPlugin.fromClass(SpellcheckLinter)
