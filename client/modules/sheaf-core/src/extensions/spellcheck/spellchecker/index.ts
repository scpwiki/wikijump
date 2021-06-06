import spellcheckerWASMRelativeURL from "spellchecker-wasm/lib/spellchecker-wasm.wasm?url"
import { Pref } from "wj-state"
import { transfer, WorkerModule } from "worker-module"
import DICTIONARIES from "./dicts"
import type { SpellcheckModuleInterface } from "./spellcheck.worker"

const spellcheckerWASMURL = new URL(
  spellcheckerWASMRelativeURL,
  import.meta.url
).toString()

async function importWorker() {
  return (await import("./spellcheck.worker?bundled-worker")).default
}

/** Class that handles and provides an interface to a spellcheck worker. */
export class SpellcheckWorker extends WorkerModule<SpellcheckModuleInterface> {
  /** The current locale of the worker. */
  declare locale: string

  /** @param locale - The locale to start the worker with. Defaults to `"en"`. */
  constructor(locale = "en") {
    super("spellchecker", importWorker, {
      persist: true,
      init: async () => {
        await this.setSpellchecker(locale)
        // add local dictionary to spellchecker once it has started
        const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
        if (localDictionary.length) {
          await this.invoke("appendToDictionary", localDictionary)
        }
      }
    })
  }

  /** Sets the locale of the worker. */
  async setSpellchecker(locale: string) {
    if (locale === this.locale) return
    if (!DICTIONARIES.hasOwnProperty(locale)) throw new Error("Invalid locale specified!")
    this.locale = locale
    const { dict, bigram } = await DICTIONARIES[locale]()
    await this.invoke("setSpellchecker", spellcheckerWASMURL, dict, bigram)
  }

  /**
   * Checks a word, and returns a list of suggestions for replacing it.
   * Returns `null` if the word isn't actually misspelled.
   */
  async check(word: string | ArrayBuffer) {
    return await this.invoke("check", transfer(word))
  }

  /**
   * Checks a arbitrary chunk of text for any misspellings. The text given
   * should not have any markup in it. Returns `null` if nothing is
   * actually misspelled.
   */
  async checkWords(str: string | ArrayBuffer) {
    return await this.invoke("checkWords", transfer(str))
  }

  /**
   * Appends a dictionary (a string) or a list of words to the current
   * spellchecker's dictionary. If directly appending a dictionary, it
   * should be noted that it is in a word frequency format.
   *
   * @example
   *
   * ```ts
   * appendtoDictionary(`
   * newword 2000
   * otherword 1000
   * `)
   * ```
   *
   * @param input - The dictionary string or word list to append.
   * @param frequency - If a list of words is provided, this will be the
   *   frequency that is set. Defaults to 1000.
   */
  async appendToDictionary(input: string | string[], frequency = 1000) {
    await this.invoke("appendToDictionary", input, frequency)
  }

  /** Saves a word to the user's internal dictionary. */
  async saveToDictionary(word: string) {
    word = word.toLowerCase()
    const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
    // add our word but do a dedupe pass to catch edge cases
    const deduped = [...new Set([...localDictionary, word])]
    Pref.set("spellchecker-user-dictionary", deduped)
    // we already appened to dictionary when the spellchecker was started
    // so we just need to add the word
    await this.invoke("appendToDictionary", [word])
  }
}

/**
 * Instance of the spellcheck worker. Will only instantiate the worker when
 * a method is first called.
 */
export const Spellchecker = new SpellcheckWorker()
