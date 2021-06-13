import { EditorSvelteComponent } from "wj-codemirror"
import type { EditorView, Tooltip } from "wj-codemirror/cm"
import { Spellcheck } from "./extension"
import MisspellingTooltip from "./MisspellingTooltip.svelte"
import type { Word } from "./types"

const tooltipHandler = new EditorSvelteComponent(MisspellingTooltip)

/** Generates a tooltip instance for a misspelling. */
export function misspelledTooltip(
  view: EditorView,
  pos: number,
  side: -1 | 1
): Tooltip | null {
  const misspellings = Spellcheck.get(view).words
  if (!misspellings.size) return null

  // given our position, find the first misspelling whose range contains our pos
  let word = null as Word | null
  misspellings.between(
    pos - (side < 0 ? 1 : 0),
    pos + (side > 0 ? 1 : 0),
    (from, to, { spec }) => {
      if (pos >= from && pos <= to) {
        word = spec.word as Word
        return false
      }
    }
  )

  if (!word) return null

  const instance = tooltipHandler.create(view, {
    pass: { word }
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
