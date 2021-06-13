import { Decoration, DecorationSet } from "wj-codemirror/cm"
import type { Word } from "./types"

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

  /** List of editor decorations for the misspelled words in the document. */
  declare readonly words: DecorationSet

  /**
   * @param enabled - Whether or not the spellchecker should be active.
   * @param locale - The locale of the spellchecker, e.g. `"en"`.
   * @param words - The list of words that the spellchecker has emitted.
   *   Can be an existing `DecorationSet` or a new array of words.
   */
  constructor(enabled: boolean, locale: string, words: Word[] | DecorationSet = []) {
    this.enabled = enabled
    this.locale = locale
    this.words = Array.isArray(words) ? Decoration.set(words.map(makeDecoration)) : words
  }

  /** Returns a new state using the given list of words. */
  set(words: Word[] | DecorationSet) {
    return new SpellcheckState(this.enabled, this.locale, words)
  }
}

/** Creates a `Range<Decoration>` for the given misspelling. */
function makeDecoration(word: Word) {
  return Decoration.mark({
    inclusive: true,
    class: "cm-misspellingRange",
    word
  }).range(word.from, word.to)
}
