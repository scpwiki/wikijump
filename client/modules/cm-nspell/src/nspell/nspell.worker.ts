import { Espells } from "espells"
import { decode, expose, ModuleProxy } from "threads-worker-module/src/worker-lib"
import type { Misspelling, Word } from ".."
import type { FlaggedWord } from "../types"

let espells: Espells

const module = {
  async set(aff: string, dic: string | string[]) {
    espells = await Espells.fromURL({ aff, dic })
  },

  // -- DICTIONARY

  async dictionary(url: string) {
    espells.addDictionary(await (await fetch(url)).text())
  },

  add(words: string | string[]) {
    if (typeof words === "string") words = [words]
    for (const word of words) {
      espells.add(word)
    }
  },

  remove(words: string | string[]) {
    if (typeof words === "string") words = [words]
    for (const word of words) {
      espells.remove(word)
    }
  },

  // -- SPELLCHECK

  correct(raw: ArrayBuffer) {
    return espells.lookup(decode(raw))
  },

  suggest(raw: ArrayBuffer, max: number) {
    return espells.suggest(decode(raw), max)
  },

  suggestions(words: Word[], max: number): Misspelling[] {
    return words.map(word => ({ ...word, suggestions: espells.suggest(word.word, max) }))
  },

  misspelled(words: Word[]) {
    return words.filter(({ word }) => !espells.lookup(word))
  },

  check(words: Word[]): FlaggedWord[] {
    return words
      .map(word => ({
        ...word,
        info: espells.lookup(word.word)
      }))
      .filter(({ info: { correct, forbidden, warn } }) => !correct || forbidden || warn)
  }
}

export type EspellsWorkerInterface = ModuleProxy<typeof module>

expose(module)
