import iterate from "iterare"
import { PriorityList } from "../plist"
import { commonCharacters, lcslen, leftCommonSubstring, lowercase, ngram } from "../util"

/** An entry in a {@link ScoresList}. */
export type ScoreEntry<T extends any[]> = [number, ...T]

/**
 * Special type of {@link PriorityList} that is intended to handle
 * suggestion scores. Stores arrays of values with a score number, and
 * limits the amount of entries within the list by removing the lowest
 * scoring values.
 *
 * @typeParam T - The array value to be stored as an entry.
 */
export class ScoresList<T extends any[]> {
  /** The comparator function that is used for the {@link PriorityList} instance. */
  private static heapCmp = (a: ScoreEntry<any>, b: ScoreEntry<any>) => a[0] - b[0]

  /**
   * The comparator function that is used when finalizing the list, which
   * requires a sort of the list.
   */
  private static finishCmp = (a: ScoreEntry<any>, b: ScoreEntry<any>) => b[0] - a[0]

  /** The internal list. */
  private list = new PriorityList<ScoreEntry<T>>(ScoresList.heapCmp)

  constructor(
    /** The maximum number of entries in the list. */
    public max: number
  ) {}

  /**
   * Adds an entry to the list.
   *
   * @param score - The score of the entry being added.
   * @param args - The entry to add.
   */
  add(score: number, ...args: T) {
    const current = this.list.peek()
    if (current && score >= current[0]) {
      this.list.push([score, ...args])
      if (this.list.length > this.max) this.list.pop()
    }
  }

  /**
   * Finalizes the list by running a sort of the list and returning a (by
   * default) normalized version of the list, so that the entries in the
   * list are as they were originally given.
   *
   * @param map - An optional mapping function that manipulates the score
   *   values before they are sorted.
   * @param keepScores - If true, the scores will not be removed from the
   *   final list of entries.
   */
  // no map, don't keep scores
  finish(map?: undefined, keepScores?: false): [...T][]
  // no map, keep scores
  finish(map?: undefined, keepScores?: true): ScoreEntry<T>[]
  // mapping function, don't keep scores
  finish<O extends any[] = T[]>(
    map: (val: ScoreEntry<T>) => ScoreEntry<O>,
    keepScores?: false
  ): [...O][]
  // mapping function, keep scores
  finish<O extends any[] = T[]>(
    map: (val: ScoreEntry<T>) => ScoreEntry<O>,
    keepScores?: true
  ): ScoreEntry<O>[]
  // actual signature
  finish<O extends any[] = T[]>(
    map?: (val: ScoreEntry<T>) => ScoreEntry<O>,
    keepScores?: boolean
  ): [...O][] | [...T][] | ScoreEntry<O>[] | ScoreEntry<T>[] {
    if (keepScores) {
      return map
        ? iterate(this.list.data).map(map).toArray().sort(ScoresList.finishCmp)
        : [...this.list.data].sort(ScoresList.finishCmp)
    } else {
      return map
        ? iterate(this.list.data)
            .map(map)
            .toArray()
            .sort(ScoresList.finishCmp)
            .map(([, ...out]) => out)
        : [...this.list.data].sort(ScoresList.finishCmp).map(([, ...out]) => out)
    }
  }
}

/**
 * Simple scoring algorithm used for determining if a potential suggestion
 * is a good one for the misspelling given.
 *
 * @param misspelling - The misspelled word.
 * @param suggestion - The potential suggestion to determine the score of.
 */
export function rootScore(misspelling: string, suggestion: string) {
  return (
    ngram(3, misspelling, lowercase(suggestion), false, false, true) +
    leftCommonSubstring(misspelling, lowercase(suggestion))
  )
}

/**
 * Simple scoring algorithm used for sorting a list of suggestions from
 * closest matching to least matching.
 *
 * @param misspelling - The misspelled word.
 * @param suggestion - The suggestion to determine the score of.
 */
export function finalScore(misspelling: string, suggestion: string) {
  return (
    2 * lcslen(misspelling, suggestion) -
    Math.abs(misspelling.length - suggestion.length) +
    leftCommonSubstring(misspelling, suggestion)
  )
}

/**
 * Finds a minimum threshold for a decent suggestion.
 *
 * @param word - The word (or misspelling) to have a threshold generated for.
 */
export function scoreThreshold(word: string) {
  let threshold = 0

  for (let startPos = 1; startPos < 4; startPos++) {
    const mangled: string[] = []
    for (let pos = startPos; pos < word.length; pos += 4) {
      mangled[pos] = "*"
    }

    const mangledWord = mangled.join("")

    threshold += ngram(word.length, word, mangledWord, false, true)
  }

  return Math.floor(threshold / (3 - 1))
}

/**
 * Simple and rough estimation of score for an affixed form.
 *
 * @param misspelling - The misspelled word.
 * @param suggestion - The suggestion to determine the score of.
 * @see {@link preciseAffixScore}
 */
export function roughAffixScore(misspelling: string, suggestion: string) {
  return (
    ngram(misspelling.length, misspelling, suggestion, false, true) +
    leftCommonSubstring(misspelling, suggestion)
  )
}

/**
 * Precise, mildly expensive (in comparison) scoring algorithm for affixed
 * forms. This function tends to generate three groups:
 *
 * - 1000 or more: The misspelling and suggestion are the same with the only
 *   exception being casing.
 * - -100 or less: The word difference is too great, as determined by
 *   `diffFactor` argument.
 * - -100...1000: Normal suggestion scores.
 *
 * @param misspelling - The misspelled word.
 * @param suggestion - The suggestion to determine the score of.
 * @param diffFactor - An adjustment knob for changing the number of
 *   suggestions returned. A lower factor means that a suggestion must be
 *   of a decent confidence to actually be given to the user.
 * @param base - The initial score between the misspelling and the suggestion.
 * @param hasPhonetic - If true, this indicates that the spellchecker also
 *   has access a {@link PhonetTable}. This causes the scores to be adjusted
 *   slightly lower so that the {@link PhonetTable} is more "important".
 */
export function preciseAffixScore(
  misspelling: string,
  suggestion: string,
  diffFactor: number,
  base: number,
  hasPhonetic: boolean
) {
  const lcs = lcslen(misspelling, suggestion)

  if (misspelling.length === suggestion.length && misspelling.length === lcs) {
    return base + 2000
  }

  let result: number

  result = 2 * lcs - Math.abs(misspelling.length - suggestion.length)

  result += leftCommonSubstring(misspelling, suggestion)

  if (commonCharacters(misspelling, lowercase(suggestion))) result++

  result += ngram(4, misspelling, suggestion, false, true)

  const bigrams =
    ngram(2, misspelling, suggestion, true, true) +
    ngram(2, suggestion, misspelling, true, true)

  result += bigrams

  let questionableLimit: number
  if (hasPhonetic) {
    questionableLimit = misspelling.length * diffFactor
  } else {
    questionableLimit = (misspelling.length + suggestion.length) * diffFactor
  }

  if (bigrams < questionableLimit) result -= 1000

  return result
}
