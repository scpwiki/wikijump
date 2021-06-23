import type { PhonetTable } from "../aff/phonet-table"
import { CONSTANTS as C } from "../constants"
import type { Word } from "../dic/word"
import { lowercase, ngram } from "../util"
import { finalScore, rootScore, ScoresList } from "./scores"

/**
 * Builder for phonetic suggestions.
 *
 * @see {@link PhonetTable}
 */
export class PhonetSuggestionBuilder {
  /** The misspelling that suggestions are being built for. */
  private declare misspelling: string

  /** A metaphone transformed version of the misspelling. */
  private declare misspellingPH: string

  /** The {@link PhonetTable} being used to build suggestions. */
  private declare table: PhonetTable

  /** The list of scores and guesses being assembled. */
  private declare scores: ScoresList<[string]>

  /**
   * @param misspelling - The misspelling to build suggestions for.
   * @param table - The {@link PhonetTable} to use.
   */
  constructor(misspelling: string, table: PhonetTable) {
    this.misspelling = misspelling
    this.misspellingPH = table.metaphone(misspelling)
    this.table = table
    this.scores = new ScoresList<[string]>(C.PHONET_MAX_ROOTS)
  }

  /**
   * Steps the builder forward by providing another {@link Word} to process.
   *
   * @param word - The {@link Word} to process.
   */
  step(word: Word) {
    if (Math.abs(word.stem.length - this.misspelling.length) > 3) return

    let nscore = rootScore(this.misspelling, word.stem)

    if (word.altSpellings?.size) {
      for (const variant of word.altSpellings) {
        nscore = Math.max(nscore, rootScore(this.misspelling, variant))
      }
    }

    if (nscore <= 2) return

    const score =
      2 *
      ngram(3, this.misspellingPH, this.table.metaphone(word.stem), false, false, true)

    this.scores.add(score, word.stem)
  }

  /** Finishes the builder and yields the resulting suggestions (as strings). */
  *finish() {
    const guesses = this.scores.finish(
      ([score, word]) =>
        [score + finalScore(this.misspelling, lowercase(word)), word] as [number, string]
    )

    for (const [suggestion] of guesses) {
      yield suggestion
    }
  }
}
