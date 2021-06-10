import { Decoration, DecorationSet } from "../../cm"
import type { Misspelling } from "./spellchecker/spellchecker"

/**
 * Holds the state of the spellchecker. Intended to be entirely immutable.
 * In order to change the list of misspellings, the
 * {@link SpellcheckState.set} method should be used, which will return a
 * new state using the given misspellings.
 */
export class SpellcheckState {
  /** True if the spellchecker should be active. */
  declare readonly enabled: boolean

  /** The current locale of the spellchecker, e.g. `"en"`. */
  declare readonly locale: string

  /** The list of editor decorations that describes the misspellings of the document. */
  declare readonly misspellings: DecorationSet

  /**
   * @param enabled - Whether or not the spellchecker should be active.
   * @param locale - The locale of the spellchecker, e.g. `"en"`.
   * @param misspellings - The list of misspellings that the spellchecker
   *   has emitted. Can be an existing `DecorationSet` or a new array of
   *   misspellings.
   */
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

  /** Returns a new state using the given list of misspellings. */
  set(misspellings: Misspelling[] | DecorationSet) {
    return new SpellcheckState(this.enabled, this.locale, misspellings)
  }
}

/** Creates a `Range<Decoration>` for the given misspelling. */
function makeDecoration(misspelling: Misspelling) {
  return Decoration.mark({
    inclusive: true,
    class: "cm-misspellingRange",
    misspelling
  }).range(misspelling.from, misspelling.to)
}
