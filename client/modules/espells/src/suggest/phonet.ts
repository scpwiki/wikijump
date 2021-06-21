import type { PhonetTable } from "../aff/phonet-table"
import { CONSTANTS as C } from "../constants"
import type { Word } from "../dic/word"
import { lowercase, ngram } from "../util"
import { finalScore, rootScore, ScoresList } from "./scores"

export class PhonetSuggestionBuilder {
  private declare misspelling: string
  private declare misspellingPH: string
  private declare table: PhonetTable
  private declare scores: ScoresList<[string]>

  constructor(misspelling: string, table: PhonetTable) {
    this.misspelling = misspelling
    this.misspellingPH = table.metaphone(misspelling)
    this.table = table
    this.scores = new ScoresList<[string]>(C.PHONET_MAX_ROOTS)
  }

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
