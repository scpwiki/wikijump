import { transfer, WorkerModule } from "threads-worker-module"
import { locale as i18nLocale, Pref } from "wj-state"
import { dedupe } from "wj-util"
import type { Word } from ".."
import DICTIONARIES from "../dicts"
import type { NSpellWorkerInterface } from "./nspell.worker"

export class NSpellWorker extends WorkerModule<NSpellWorkerInterface> {
  declare locale: string
  declare disabled: boolean

  constructor(locale = "en") {
    super("nspell", importWorker, {
      persist: true,
      init: async () => void (await this.set(locale, true))
    })
    this.disabled = false
    this.locale = localeLanguage(locale)
  }

  async set(locale: string, force = false) {
    locale = localeLanguage(locale)
    if (!force && this.locale === locale) return

    if (DICTIONARIES.hasOwnProperty(locale)) {
      const { aff, dic } = await DICTIONARIES[locale]()
      await this.invoke("set", aff, dic)

      // add local dictionary to spellchecker once it has started
      const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
      if (localDictionary.length) {
        await this.invoke("personal", localDictionary)
      }

      this.locale = locale
      this.disabled = false
    } else {
      console.warn("Locale given to spellchecker has no resources available for it.")
      this.disabled = true
    }
  }

  async dictionary(url: string) {
    if (this.disabled) return
    await this.invoke("dictionary", url)
  }

  async personal(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("personal", words)
  }

  async add(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("add", words)
  }

  async remove(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("remove", words)
  }

  async saveToDictionary(word: string) {
    if (this.disabled) return
    const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
    // add our word but do a dedupe pass to catch edge cases
    Pref.set("spellchecker-user-dictionary", dedupe(localDictionary, word))
    // we already added everything to the dictionary when the spellchecker was started
    // so we only need to add the word that was just added
    await this.invoke("add", word)
  }

  async correct(word: string) {
    if (this.disabled) return true
    return await this.invoke("correct", transfer(word))
  }

  async info(word: string) {
    if (this.disabled) return { correct: true, forbidden: false, warn: false }
    return await this.invoke("info", transfer(word))
  }

  async suggest(word: string) {
    if (this.disabled) return []
    return await this.invoke("suggest", transfer(word))
  }

  async check(word: string) {
    if (this.disabled) return null
    return await this.invoke("check", transfer(word))
  }

  async suggestions(words: Word[]) {
    if (this.disabled) return []
    return await this.invoke("suggestions", words)
  }

  async misspelled(words: Word[]) {
    if (this.disabled) return []
    return await this.invoke("misspelled", words)
  }

  async wordCharacters() {
    if (this.disabled) return null
    return await this.invoke("wordCharacters")
  }
}

function localeLanguage(locale: string) {
  return locale.toLowerCase().split(/-|_/)[0]
}

export default new NSpellWorker(i18nLocale)

async function importWorker() {
  return (await import("./nspell.worker?bundled-worker")).default
}
