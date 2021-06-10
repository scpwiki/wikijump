import spellcheckerWASMRelativeURL from "spellchecker-wasm/lib/spellchecker-wasm.wasm?url"
import { transfer, WorkerModule } from "threads-worker-module"
import { locale as i18nLocale, Pref } from "wj-state"
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
  /** True if the worker is disabled due to a locale that isn't available. */
  declare disabled: boolean

  /** The current locale of the worker. */
  declare locale: string

  /** @param locale - The locale to start the worker with. Defaults to `"en"`. */
  constructor(locale = "en") {
    super("spellchecker", importWorker, {
      persist: true,
      init: async () => await this.setSpellchecker(locale)
    })
  }

  /** Sets the locale of the worker. */
  async setSpellchecker(locale: string) {
    if (locale === this.locale) return
    if (DICTIONARIES.hasOwnProperty(locale)) {
      this.disabled = false
      this.locale = locale
      const { dict, bigram } = await DICTIONARIES[locale]()
      const urls = { wasm: spellcheckerWASMURL, dict, bigram }
      await this.invoke("setSpellchecker", locale, urls)
      // add local dictionary to spellchecker once it has started
      const localDictionary = Pref.get<string[]>("spellchecker-user-dictionary", [])
      if (localDictionary.length) {
        await this.invoke("appendToDictionary", localDictionary)
      }
    } else {
      console.warn("Locale given to spellchecker has no resources available for it.")
      this.disabled = true
    }
  }

  /**
   * Checks a word, and returns a list of suggestions for replacing it.
   * Returns `null` if the word isn't actually misspelled.
   */
  async check(word: string | ArrayBuffer) {
    if (this.disabled) return null
    return await this.invoke("check", transfer(word))
  }

  /**
   * Checks a arbitrary chunk of text for any misspellings. The text given
   * should not have any markup in it. Returns `null` if nothing is
   * actually misspelled.
   */
  async checkWords(str: string | ArrayBuffer) {
    if (this.disabled) return null
    return await this.invoke("checkWords", transfer(str))
  }

  /**
   * Segments a given string of string of text. Simply returns the
   * spellchecker's best guess on how the string should be segmented, while
   * attempting to not change any of the spelling.
   *
   * @param str - The string to be segmented.
   */
  async segment(str: string | ArrayBuffer) {
    if (this.disabled) return typeof str === "string" ? str : decode(str)
    return decode(await this.invoke("segment", transfer(str)))
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
    if (this.disabled) return
    await this.invoke("appendToDictionary", input, frequency)
  }

  /** Saves a word to the user's internal dictionary. */
  async saveToDictionary(word: string) {
    if (this.disabled) return
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

// get the locale, but rip off any region codes
// not quite sure how this will always be formatted,
// so this is done in a paranoid fashion
const [locale] = i18nLocale.toLowerCase().split(/-|_/)

/**
 * Instance of the spellcheck worker. Will only instantiate the worker when
 * a method is first called.
 */
export const Spellchecker = new SpellcheckWorker(locale)
