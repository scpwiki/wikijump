import type { Readable, Writable } from "svelte/store"
import type { SheafCore } from "./core"
import type { SheafBindings } from "./extensions/bindings"

export interface SheafSettings {
  debug: boolean
  editor: {
    darkmode: boolean
    spellcheck: boolean
  }
  preview: {
    enabled: boolean
    darkmode: boolean
  }
}

export interface SheafContext {
  editor: SheafCore
  bindings: SheafBindings
  settings: Writable<SheafSettings>
  small: Readable<boolean>
}

export function getDefaultSheafSettings(): SheafSettings {
  return {
    debug: false,
    editor: {
      darkmode: true,
      spellcheck: true
    },
    preview: {
      enabled: true,
      darkmode: false
    }
  }
}
