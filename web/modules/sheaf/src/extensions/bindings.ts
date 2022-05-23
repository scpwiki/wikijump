import type { Extension } from "@codemirror/state"
import { keymap, ViewPlugin } from "@codemirror/view"
import { debounce } from "@wikijump/util"
import type { SheafCore } from "../core"

export interface SheafBindings {
  /** Callback fired when the user "saves", e.g. hitting `CTRL + S`. */
  save?: (core: SheafCore) => void

  /**
   * Callback fired when the document state changes. _This is debounced._
   * It won't be called immediately after a change.
   */
  update?: (core: SheafCore) => void
}

export function createSheafBinding(core: SheafCore, bindings: SheafBindings) {
  const extensions: Extension[] = []

  if (bindings.save) {
    extensions.push(
      keymap.of([
        { key: "Mod-S", run: () => (bindings.save!(core), true), preventDefault: true }
      ])
    )
  }

  if (bindings.update) {
    const callback = debounce(bindings.update, 50)
    extensions.push(
      ViewPlugin.define(() => ({
        update: () => callback(core)
      }))
    )
  }

  return extensions
}
