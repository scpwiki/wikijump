import iterate from "iterare"
import { PriorityList } from "../plist"
import { leftCommonSubstring, lowercase, ngram } from "../util"

export type ScoreEntry<T extends any[]> = [number, ...T]

export class ScoresList<T extends any[]> {
  static heapCmp = (a: ScoreEntry<any>, b: ScoreEntry<any>) => a[0] - b[0]
  static finishCmp = (a: ScoreEntry<any>, b: ScoreEntry<any>) => b[0] - a[0]

  list = new PriorityList<ScoreEntry<T>>(ScoresList.heapCmp)

  constructor(public max: number) {}

  add(score: number, ...args: T) {
    const current = this.list.peek()
    if (current && score >= current[0]) {
      this.list.push([score, ...args])
      if (this.list.length > this.max) this.list.pop()
    }
  }

  finish(map?: undefined, keepScores?: false): [...T][]
  finish(map?: undefined, keepScores?: true): ScoreEntry<T>[]
  finish<O extends any[] = T[]>(
    map: (val: ScoreEntry<T>) => ScoreEntry<O>,
    keepScores?: false
  ): [...O][]
  finish<O extends any[] = T[]>(
    map: (val: ScoreEntry<T>) => ScoreEntry<O>,
    keepScores?: true
  ): ScoreEntry<O>[]
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
