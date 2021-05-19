import type { SheafCore } from "sheaf-core"
import type { SheafBindings } from "sheaf-core/src/bindings"
import type { Writable } from "svelte/store"

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
