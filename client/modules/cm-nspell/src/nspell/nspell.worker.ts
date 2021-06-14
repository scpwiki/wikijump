import NSpell from "nspell"
import { decode, expose, ModuleProxy } from "threads-worker-module/src/worker-lib"
import type { Misspelling, Word } from ".."
import type { FlaggedWord } from "../types"

let nspell: NSpell

const module = {
  async set(affURL: string, dictURL: string | string[]) {
    if (Array.isArray(dictURL)) {
      const aff = await fetchText(affURL)
      const dictionaries: { aff: string; dic: string }[] = []
      for (const url of dictURL) {
        const dic = await fetchText(url)
        dictionaries.push({ aff, dic })
      }
      nspell = NSpell(dictionaries)
    } else {
      nspell = NSpell(await fetchText(affURL), await fetchText(dictURL))
    }
  },

  // -- DICTIONARY

  async dictionary(url: string) {
    nspell.dictionary(await fetchText(url))
  },

  personal(words: string | string[]) {
    if (Array.isArray(words)) words = words.join("\n")
    nspell.personal(words)
  },

  add(words: string | string[]) {
    if (typeof words === "string") words = [words]
    for (const word of words) {
      nspell.add(word)
    }
  },

  remove(words: string | string[]) {
    if (typeof words === "string") words = [words]
    for (const word of words) {
      nspell.remove(word)
    }
  },

  // -- SPELLCHECK

  correct(raw: ArrayBuffer) {
    return nspell.correct(decode(raw))
  },

  info(raw: ArrayBuffer) {
    return nspell.spell(decode(raw))
  },

  suggest(raw: ArrayBuffer, max: number) {
    return clampedSuggest(decode(raw), max)
  },

  suggestions(words: Word[], max: number): Misspelling[] {
    return words.map(word => ({ ...word, suggestions: clampedSuggest(word.word, max) }))
  },

  misspelled(words: Word[]) {
    return words.filter(({ word }) => !nspell.correct(word))
  },

  check(words: Word[]): FlaggedWord[] {
    return words
      .map(word => ({ ...word, info: nspell.spell(word.word) }))
      .filter(({ info: { correct, forbidden, warn } }) => !correct || forbidden || warn)
  },

  // -- MISC

  wordCharacters() {
    return nspell.wordCharacters() ?? null
  }
}

async function fetchText(url: string) {
  return await (await fetch(url)).text()
}

function clampedSuggest(word: string, max: number) {
  let suggestions = nspell.suggest(word)
  if (max && suggestions.length > max) suggestions = suggestions.slice(0, max)
  return suggestions
}

export type NSpellWorkerInterface = ModuleProxy<typeof module>

expose(module)
