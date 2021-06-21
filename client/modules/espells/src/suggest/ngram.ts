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

export function* ngramSuggest(
  misspelling: string,
  dictionaryWords: Set<Word>,
  prefixes: PrefixMap,
  suffixes: SuffixMap,
  known: Set<string>,
  maxDiff: number,
  onlyMaxDiff = false,
  hasPhonetic = false
) {
  const roots = new ScoresList<[Word]>(C.NGRAM_MAX_ROOTS)

  for (const word of dictionaryWords) {
    if (Math.abs(word.stem.length - misspelling.length) > 4) continue

    let score = rootScore(misspelling, word.stem)

    if (word.altSpellings?.size) {
      for (const variant of word.altSpellings) {
        score = Math.max(score, rootScore(misspelling, variant))
      }
    }

    roots.add(score, word)
  }

  const threshold = scoreThreshold(misspelling)

  const guesses = new ScoresList<[string, string]>(C.NGRAM_MAX_GUESSES)

  for (const [root] of roots.finish()) {
    if (root.altSpellings?.size) {
      for (const variant of root.altSpellings) {
        const score = roughAffixScore(misspelling, variant)
        if (score > threshold) guesses.add(score, variant, root.stem)
      }
    }

    for (const form of formsFor(root, prefixes, suffixes, misspelling)) {
      const score = roughAffixScore(misspelling, lowercase(form))
      if (score > threshold) guesses.add(score, form, form)
    }
  }

  const fact = maxDiff >= 0 ? (10 - maxDiff) / 5 : 1

  const guesses2 = guesses.finish(
    ([score, compared, real]) =>
      [
        preciseAffixScore(misspelling, lowercase(compared), fact, score, hasPhonetic),
        real
      ] as [number, string],
    true
  )

  yield* filterGuesses(guesses2, known, onlyMaxDiff)
}

function formsFor(
  word: Word,
  allPrefixes: PrefixMap,
  allSuffixes: SuffixMap,
  similarTo: string
) {
  const res: string[] = [word.stem]

  const suffixes = !word.flags
    ? []
    : iterate(word.flags)
        .filter(flag => allSuffixes.has(flag))
        .map(flag => allSuffixes.get(flag)!)
        .flatten()
        .filter(suffix => suffix.relevant(word.stem) && similarTo.endsWith(suffix.add))
        .toArray()

  const prefixes = !word.flags
    ? []
    : iterate(word.flags)
        .filter(flag => allPrefixes.has(flag))
        .map(flag => allPrefixes.get(flag)!)
        .flatten()
        .filter(prefix => prefix.relevant(word.stem) && similarTo.startsWith(prefix.add))
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

function* filterGuesses(
  guesses: [number, string][],
  known: Set<string>,
  onlyMaxDiff = true
) {
  let seen = false
  let found = 0

  for (const [score, value] of guesses) {
    if (seen && score <= 1000) return

    if (score > 1000) seen = true
    else if (score < -100) {
      if (found > 0 || onlyMaxDiff) return
      seen = true
    }

    if (!iterate(known).some(word => word.includes(value))) {
      found++
      yield value
    }
  }
}
