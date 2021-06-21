import iterate from "iterare"
import type { PrefixMap, SuffixMap } from "../aff"
import type { Prefix, Suffix } from "../aff/affix"
import { CONSTANTS as C } from "../constants"
import type { Word } from "../dic/word"
import { lowercase } from "../util"
import {
  preciseAffixScore,
  rootScore,
  roughAffixScore,
  ScoresList,
  scoreThreshold
} from "./scores"

export class NgramSuggestionBuilder {
  private declare misspelling: string
  private declare prefixes: PrefixMap
  private declare suffixes: SuffixMap
  private declare known: Set<string>
  private declare maxDiff: number
  private declare onlyMaxDiff: boolean
  private declare hasPhonetic: boolean
  private declare roots: ScoresList<[Word]>

  constructor(
    misspelling: string,
    prefixes: PrefixMap,
    suffixes: SuffixMap,
    known: Set<string>,
    maxDiff: number,
    onlyMaxDiff = false,
    hasPhonetic = false
  ) {
    this.misspelling = misspelling
    this.prefixes = prefixes
    this.suffixes = suffixes
    this.known = known
    this.maxDiff = maxDiff
    this.onlyMaxDiff = onlyMaxDiff
    this.hasPhonetic = hasPhonetic
    this.roots = new ScoresList<[Word]>(C.NGRAM_MAX_ROOTS)
  }

  step(word: Word) {
    if (Math.abs(word.stem.length - this.misspelling.length) > 4) return

    let score = rootScore(this.misspelling, word.stem)

    if (word.altSpellings?.size) {
      for (const variant of word.altSpellings) {
        score = Math.max(score, rootScore(this.misspelling, variant))
      }
    }

    this.roots.add(score, word)
  }

  *finish() {
    const threshold = scoreThreshold(this.misspelling)

    const guesses = new ScoresList<[string, string]>(C.NGRAM_MAX_GUESSES)

    for (const [root] of this.roots.finish()) {
      if (root.altSpellings?.size) {
        for (const variant of root.altSpellings) {
          const lower = lowercase(variant)
          const score = roughAffixScore(this.misspelling, variant)
          if (score > threshold) guesses.add(score, lower, root.stem)
        }
      }

      for (const form of this.formsFor(root, this.misspelling)) {
        const lower = lowercase(form)
        const score = roughAffixScore(this.misspelling, form)
        if (score > threshold) guesses.add(score, lower, form)
      }
    }

    const fact = this.maxDiff >= 0 ? (10 - this.maxDiff) / 5 : 1

    yield* this.filterGuesses(
      guesses.finish(
        ([score, compared, real]) =>
          [
            preciseAffixScore(this.misspelling, compared, fact, score, this.hasPhonetic),
            real
          ] as [number, string],
        true
      )
    )
  }

  formsFor(word: Word, similarTo: string) {
    const res: string[] = [word.stem]

    const suffixes = !word.flags
      ? []
      : iterate(word.flags)
          .filter(flag => this.suffixes.has(flag))
          .map(flag => this.suffixes.get(flag)!)
          .flatten()
          .filter(suffix => suffix.relevant(word.stem) && similarTo.endsWith(suffix.add))
          .toArray()

    const prefixes = !word.flags
      ? []
      : iterate(word.flags)
          .filter(flag => this.suffixes.has(flag))
          .map(flag => this.suffixes.get(flag)!)
          .flatten()
          .filter(
            prefix => prefix.relevant(word.stem) && similarTo.startsWith(prefix.add)
          )
          .toArray()

    const cross = iterate(prefixes)
      .map(prefix =>
        iterate(suffixes)
          .filter(suffix => suffix.crossproduct && prefix.crossproduct)
          .map(suffix => [prefix, suffix] as [Prefix, Suffix])
          .toArray()
      )
      .flatten()
      .toArray()

    for (const suffix of suffixes) {
      const root = suffix.strip ? word.stem.slice(0, -suffix.strip.length) : word.stem
      res.push(root + suffix.add)
    }

    for (const [prefix, suffix] of cross) {
      const root = suffix.strip
        ? word.stem.slice(prefix.strip.length, -suffix.strip.length)
        : word.stem.slice(prefix.strip.length)
      res.push(prefix.add + root + suffix.add)
    }

    for (const prefix of prefixes) {
      const root = word.stem.slice(prefix.strip.length)
      res.push(prefix.add + root)
    }

    return res
  }

  *filterGuesses(guesses: [number, string][]) {
    let seen = false
    let found = 0

    for (const [score, value] of guesses) {
      if (seen && score <= 1000) return

      if (score > 1000) seen = true
      else if (score < -100) {
        if (found > 0 || this.onlyMaxDiff) return
        seen = true
      }

      if (!iterate(this.known).some(word => word.includes(value))) {
        found++
        yield value
      }
    }
  }
}
