import { Decoration, DecorationSet } from "@codemirror/view"
import type { Misspelling } from "./spellchecker/spellchecker"

export class SpellcheckState {
  declare readonly enabled: boolean
  declare readonly locale: string
  declare readonly misspellings: DecorationSet

  constructor(
    enabled: boolean,
    locale: string,
    misspellings: Misspelling[] | DecorationSet = []
  ) {
    this.enabled = enabled
    this.locale = locale
    this.misspellings = Array.isArray(misspellings)
      ? Decoration.set(misspellings.map(makeDecoration))
      : misspellings
  }

  set(misspellings: Misspelling[] | DecorationSet) {
    return new SpellcheckState(this.enabled, this.locale, misspellings)
  }
}

function makeDecoration(misspelling: Misspelling) {
  return Decoration.mark({
    inclusive: true,
    class: "cm-misspellingRange",
    misspelling
  }).range(misspelling.from, misspelling.to)
}
