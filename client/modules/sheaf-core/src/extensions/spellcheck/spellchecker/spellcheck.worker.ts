import { decode, expose, ModuleProxy } from "worker-module/src/worker-lib"
import * as Spellchecker from "./spellchecker"

const module = {
  async setSpellchecker(wasmURL: string, dictURL: string, bigramURL?: string) {
    await Spellchecker.setSpellchecker(wasmURL, dictURL, bigramURL)
  },

  async spellcheck(raw: ArrayBuffer) {
    return await Spellchecker.check(decode(raw))
  },

  async spellcheckWords(raw: ArrayBuffer) {
    return await Spellchecker.checkWords(decode(raw))
  },

  appendToDictionary(input: string | string[], frequency = 1000) {
    Spellchecker.appendToDictionary(input, frequency)
  }
}

export type SpellcheckModuleInterface = ModuleProxy<typeof module>

expose(module)
