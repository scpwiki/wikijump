import { Decoration, type DecorationSet } from "@wikijump/codemirror/cm"
import type { FlaggedWord } from "./types"

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

  /** List of editor decorations for the flagged words in the document. */
  declare readonly flagged: DecorationSet

  /**
   * @param enabled - Whether or not the spellchecker should be active.
   * @param locale - The locale of the spellchecker, e.g. `"en"`.
   * @param flagged - The list of words that the spellchecker has flagged.
   *   Can be an existing `DecorationSet` or a new array of words.
   */
  constructor(
    enabled: boolean,
    locale: string,
    flagged: FlaggedWord[] | DecorationSet = []
  ) {
    this.enabled = enabled
    this.locale = locale
    this.flagged = Array.isArray(flagged)
      ? Decoration.set(flagged.map(makeDecoration))
      : flagged
  }

  /**
   * Returns a new state using the given list of words.
   *
   * - @param flagged - The list of words that the spellchecker has flagged.
   *   Can be an existing `DecorationSet` or a new array of words.
   */
  set(flagged: FlaggedWord[] | DecorationSet) {
    return new SpellcheckState(this.enabled, this.locale, flagged)
  }
}

/** Creates a `Range<Decoration>` for the given flagged word. */
function makeDecoration(word: FlaggedWord) {
  // assemble classes depending on how the word is flagged
  const classes = ["cm-spellcheckRange"]
  if (!word.info.correct) classes.push("cm-spellcheckRange-misspelled")
  if (word.info.forbidden) classes.push("cm-spellcheckRange-forbidden")
  if (word.info.warn) classes.push("cm-spellcheckRange-warn")

  return Decoration.mark({
    inclusive: true,
    class: classes.join(" "),
    word
  }).range(word.from, word.to)
}
