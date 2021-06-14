import NSpell from "nspell"
import { decode, expose, ModuleProxy } from "threads-worker-module/src/worker-lib"
import type { Misspelling, Word } from ".."

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

  suggest(raw: ArrayBuffer) {
    return nspell.suggest(decode(raw))
  },

  check(raw: ArrayBuffer) {
    const word = decode(raw)
    if (nspell.correct(word)) return null
    return nspell.suggest(word)
  },

  suggestions(words: Word[]): Misspelling[] {
    return words.map(word => ({ ...word, suggestions: nspell.suggest(word.word) }))
  },

  misspelled(words: Word[]) {
    return words.filter(({ word }) => !nspell.correct(word))
  },

  // -- MISC

  wordCharacters() {
    return nspell.wordCharacters() ?? null
  }
}

async function fetchText(url: string) {
  return await (await fetch(url)).text()
}

export type NSpellWorkerInterface = ModuleProxy<typeof module>

expose(module)
