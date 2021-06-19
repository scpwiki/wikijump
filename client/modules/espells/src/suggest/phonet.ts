import type { PhonetTable } from "../aff/phonet-table"
import type { Word } from "../dic/word"
import { lcslen, leftCommonSubstring, lowercase, ngram, uppercase } from "../util"
import { rootScore } from "./ngram"
import { ScoresList } from "./scores"

const MAX_ROOTS = 100

export function* phonetSuggest(
  misspelling: string,
  dictionaryWords: Set<Word>,
  table: PhonetTable
) {
  misspelling = lowercase(misspelling)
  const misspelling_ph = metaphone(table, misspelling)

  const scores = new ScoresList<[string]>(MAX_ROOTS)

  for (const word of dictionaryWords) {
    if (Math.abs(word.stem.length - misspelling.length) > 3) continue

    let nscore = rootScore(misspelling, word.stem)

    if (word.altSpellings?.size) {
      for (const variant of word.altSpellings) {
        nscore = Math.max(nscore, rootScore(misspelling, variant))
      }
    }

    if (nscore <= 2) continue

    const score =
      2 * ngram(3, misspelling_ph, metaphone(table, word.stem), false, false, true)

    scores.add(score, word.stem)
  }

  const guesses = scores.finish(
    ([score, word]) =>
      [score + finalScore(misspelling, lowercase(word)), word] as [number, string]
  )

  for (const [suggestion] of guesses) {
    yield suggestion
  }
}

function finalScore(word1: string, word2: string) {
  return (
    2 * lcslen(word1, word2) -
    Math.abs(word1.length - word2.length) +
    leftCommonSubstring(word1, word2)
  )
}

function metaphone(table: PhonetTable, word: string) {
  word = uppercase(word)
  let pos = 0
  let res = ""

  while (pos < word.length) {
    let match: false | RegExpExecArray = false
    if (table.rules[word[pos]]) {
      for (const rule of table.rules[word[pos]]) {
        match = rule.match(word, pos)
        if (match) {
          res += rule.replacement
          pos += match.index! + match[0].length - match.index!
        }
      }
    }
    if (!match) pos++
  }

  return res
}
