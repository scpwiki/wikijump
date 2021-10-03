import {
  decode,
  expose,
  ModuleProxy
} from "@wikijump/threads-worker-module/src/worker-lib"
import { Espells, OverridableAffData } from "espells"
import type { Misspelling, Word } from ".."
import type { FlaggedWord } from "../types"

let espells: Espells

const module = {
  async set(aff: string, dic: string | string[], override?: OverridableAffData) {
    espells = await Espells.fromURL({ aff, dic, override })
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

  lookup(raw: ArrayBuffer, caseSensitive?: boolean) {
    return espells.lookup(decode(raw), caseSensitive)
  },

  suggest(raw: ArrayBuffer, max: number) {
    return espells.suggest(decode(raw), max)
  },

  suggestions(words: Word[], max: number): Misspelling[] {
    return words.map(word => ({ ...word, suggestions: espells.suggest(word.word, max) }))
  },

  misspelled(words: Word[], caseSensitive?: boolean) {
    return words.filter(({ word }) => !espells.lookup(word, caseSensitive))
  },

  check(words: Word[], caseSensitive?: boolean): FlaggedWord[] {
    return words
      .map(word => ({
        ...word,
        info: espells.lookup(word.word, caseSensitive)
      }))
      .filter(({ info: { correct, forbidden, warn } }) => !correct || forbidden || warn)
  },

  // -- MISC

  stems(raw: ArrayBuffer, caseSensitive?: boolean) {
    return espells.stems(decode(raw), caseSensitive)
  },

  data(raw: ArrayBuffer, caseSensitive?: boolean) {
    return espells.data(decode(raw), caseSensitive)
  }
}

export type EspellsWorkerInterface = ModuleProxy<typeof module>

expose(module)
