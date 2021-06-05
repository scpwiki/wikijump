import type { Tooltip } from "@codemirror/tooltip"
import type { EditorView } from "@codemirror/view"
import { EditorSvelteComponent } from "../../adapters/svelte-dom"
import { Spellcheck } from "./extension"
import MisspellingTooltip from "./MisspellingTooltip.svelte"
import type { Misspelling } from "./spellchecker/spellchecker"

const tooltipHandler = new EditorSvelteComponent(MisspellingTooltip)

export function misspelledTooltip(
  view: EditorView,
  pos: number,
  side: -1 | 1
): Tooltip | null {
  const misspellings = Spellcheck.get(view).misspellings
  if (!misspellings.size) return null

  let word = null as Misspelling | null
  misspellings.between(
    pos - (side < 0 ? 1 : 0),
    pos + (side > 0 ? 1 : 0),
    (from, to, { spec }) => {
      if (pos >= from && pos <= to) {
        word = spec.misspelling as Misspelling
        return false
      }
    }
  )

  if (!word) return null

  const instance = tooltipHandler.create(view, {
    pass: { misspelling: word }
  })

  return {
    pos: word.from,
    end: word.to,
    create: () => ({
      dom: instance.dom,
      update: update => instance.update(update)
    })
  }
}
