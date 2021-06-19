import iterate from "iterare"
import type { PrefixMap, SuffixMap } from "../aff"
import type { Prefix, Suffix } from "../aff/affix"
import { CONSTANTS as C } from "../constants"
import type { Word } from "../dic/word"
import { commonCharacters, lcslen, leftCommonSubstring, lowercase, ngram } from "../util"
import { ScoresList } from "./scores"

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

  const threshold = detectThreshold(misspelling)

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

export function rootScore(word1: string, word2: string) {
  return (
    ngram(3, word1, lowercase(word2), false, false, true) +
    leftCommonSubstring(word1, lowercase(word2))
  )
}

function roughAffixScore(word1: string, word2: string) {
  return (
    ngram(word1.length, word1, word2, false, true) + leftCommonSubstring(word1, word2)
  )
}

function preciseAffixScore(
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

function detectThreshold(word: string) {
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
        .filter(
          suffix =>
            suffix.conditionRegex.test(word.stem) && similarTo.endsWith(suffix.add)
        )
        .toArray()

  const prefixes = !word.flags
    ? []
    : iterate(word.flags)
        .filter(flag => allPrefixes.has(flag))
        .map(flag => allPrefixes.get(flag)!)
        .flatten()
        .filter(
          prefix =>
            prefix.conditionRegex.test(word.stem) && similarTo.startsWith(prefix.add)
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
