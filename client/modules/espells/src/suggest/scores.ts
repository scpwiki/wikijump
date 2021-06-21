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

export function rootScore(word1: string, word2: string) {
  return (
    ngram(3, word1, lowercase(word2), false, false, true) +
    leftCommonSubstring(word1, lowercase(word2))
  )
}

export function finalScore(word1: string, word2: string) {
  return (
    2 * lcslen(word1, word2) -
    Math.abs(word1.length - word2.length) +
    leftCommonSubstring(word1, word2)
  )
}

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

export function roughAffixScore(word1: string, word2: string) {
  return (
    ngram(word1.length, word1, word2, false, true) + leftCommonSubstring(word1, word2)
  )
}

export function preciseAffixScore(
  word1: string,
  word2: string,
  diffFactor: number,
  base: number,
  hasPhonetic: boolean
) {
  const lcs = lcslen(word1, word2)

  if (word1.length === word2.length && word1.length === lcs) {
    return base + 2000
  }

  let result: number

  result = 2 * lcs - Math.abs(word1.length - word2.length)

  result += leftCommonSubstring(word1, word2)

  if (commonCharacters(word1, lowercase(word2))) result++

  result += ngram(4, word1, word2, false, true)

  const bigrams = ngram(2, word1, word2, true, true) + ngram(2, word2, word1, true, true)

  result += bigrams

  let questionableLimit: number
  if (hasPhonetic) {
    questionableLimit = word1.length * diffFactor
  } else {
    questionableLimit = (word1.length + word2.length) * diffFactor
  }

  if (bigrams < questionableLimit) result -= 1000

  return result
}
