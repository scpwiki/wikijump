import {
  decode,
  expose,
  ModuleProxy,
  transfer
} from "threads-worker-module/src/worker-lib"
import { Spellchecker } from "./spellchecker"
import type { SpellcheckerOptions, SpellcheckerURLS } from "./types"

let spellchecker: Spellchecker

const module = {
  async setSpellchecker(
    locale: string,
    urls: SpellcheckerURLS,
    options?: SpellcheckerOptions
  ) {
    if (spellchecker) spellchecker.free()
    spellchecker = new Spellchecker(locale, urls, options)
  },

  async check(raw: ArrayBuffer) {
    return await spellchecker.check(decode(raw))
  },

  async checkSentence(raw: ArrayBuffer) {
    return await spellchecker.checkSentence(decode(raw))
  },

  async checkWords(raw: ArrayBuffer) {
    return await spellchecker.checkWords(decode(raw))
  },

  async segment(raw: ArrayBuffer) {
    return transfer(await spellchecker.segment(decode(raw)))
  },

  async appendToDictionary(input: string | string[], frequency = 1000) {
    await spellchecker.appendToDictionary(input, frequency)
  }
}

export type SpellcheckModuleInterface = ModuleProxy<typeof module>

expose(module)
