import spellcheckerWASMRelativeURL from "spellchecker-wasm/lib/spellchecker-wasm.wasm?url"
import { Pref } from "wj-state"
import { transfer, WorkerModule } from "worker-module"
import dicts from "./dicts"
import type { SpellcheckModuleInterface } from "./spellcheck.worker"

const spellcheckerWASMURL = new URL(
  spellcheckerWASMRelativeURL,
  import.meta.url
).toString()

async function importWorker() {
  return (await import("./spellcheck.worker?bundled-worker")).default
}

export class SpellcheckWorker extends WorkerModule<SpellcheckModuleInterface> {
  declare locale: string

  constructor() {
    super("spellchecker", importWorker, {
      persist: true,
      init: async () => {
        const { dict, bigram } = await dicts.en()
        await this.invoke("setSpellchecker", spellcheckerWASMURL, dict, bigram)
        const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
        if (localDictionary.length) {
          await this.invoke("appendToDictionary", localDictionary)
        }
      }
    })

    this.locale = "en"
  }

  async setSpellchecker(locale: string) {
    if (locale === this.locale) return
    if (!dicts.hasOwnProperty(locale)) throw new Error("Invalid locale specified!")
    this.locale = locale
    const { dict, bigram } = await dicts[locale]()
    await this.invoke("setSpellchecker", spellcheckerWASMURL, dict, bigram)
  }

  async spellcheck(word: string | ArrayBuffer) {
    return await this.invoke("spellcheck", transfer(word))
  }

  async spellcheckWords(str: string | ArrayBuffer) {
    return await this.invoke("spellcheckWords", transfer(str))
  }

  async appendToDictionary(input: string | string[], frequency = 1000) {
    await this.invoke("appendToDictionary", input, frequency)
  }

  async saveToDictionary(word: string) {
    const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
    // add our word but do a dedupe pass to catch edge cases
    const deduped = [...new Set([...localDictionary, word])]
    Pref.set("spellchecker-user-dictionary", deduped)
    // we already appened to dictionary when the spellchecker was started
    // so we just need to add the word
    await this.invoke("appendToDictionary", [word])
  }
}

export const Spellchecker = new SpellcheckWorker()
