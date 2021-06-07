import { decode, expose, ModuleProxy } from "threads-worker-module/src/worker-lib"
import { Spellchecker, SpellcheckerURLS } from "./spellchecker"

let spellchecker: Spellchecker

const module = {
  async setSpellchecker(locale: string, urls: SpellcheckerURLS) {
    spellchecker = new Spellchecker(locale, urls)
  },

  async check(raw: ArrayBuffer) {
    return await spellchecker.check(decode(raw))
  },

  async checkWords(raw: ArrayBuffer) {
    return await spellchecker.checkWords(decode(raw))
  },

  async appendToDictionary(input: string | string[], frequency = 1000) {
    await spellchecker.appendToDictionary(input, frequency)
  }
}

export type SpellcheckModuleInterface = ModuleProxy<typeof module>

expose(module)
