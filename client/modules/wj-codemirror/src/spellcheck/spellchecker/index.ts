import { decode, transfer, WorkerModule } from "threads-worker-module"
import { locale as i18nLocale, Pref } from "wj-state"
import { lowercase } from "wj-util"
import symSpellRelativeURL from "../../../vendor/symspell_bg.wasm?url"
import DICTIONARIES from "./dicts"
import type { SpellcheckModuleInterface } from "./spellcheck.worker"

const symSpellURL = new URL(symSpellRelativeURL, import.meta.url).toString()

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
    ;[locale] = locale.toLowerCase().split(/-|_/)
    if (!DICTIONARIES.hasOwnProperty(locale)) {
      console.warn("Locale given to spellchecker has no resources available for it.")
      this.disabled = true
    }
  }

  /**
   * Sets the locale of the worker.
   *
   * @param locale - The locale to use. Only the language part of the
   *   locale given will be used.
   */
  async setSpellchecker(locale: string) {
    if (locale === this.locale) return // split out the language code, discard region code
    ;[locale] = locale.toLowerCase().split(/-|_/)

    if (DICTIONARIES.hasOwnProperty(locale)) {
      this.disabled = false
      this.locale = locale
      const { dict, bigram } = await DICTIONARIES[locale]()
      const urls = { wasm: symSpellURL, dict, bigram }
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
   * Checks the spelling of a word. Returns `null` if nothing is misspelled.
   *
   * @param word - The word to spellcheck.
   */
  async check(word: string | ArrayBuffer) {
    if (this.disabled) return null
    return await this.invoke("check", transfer(word))
  }

  /**
   * Checks the spelling of a sentence. Attempts to segment the sentence
   * given into a reasonable set, which accounts compound words and other
   * forms of unsegmented text. Returns `null` if the sentence has nothing
   * to correct.
   *
   * @param sentence - The sentence to segment and then spellcheck.
   */
  async checkSentence(sentence: string | ArrayBuffer) {
    if (this.disabled) return null
    return await this.invoke("checkSentence", transfer(sentence))
  }

  /**
   * Finds the words of a string and then runs the spellchecker on each
   * one. Returns a list of words that the spellchecker believes to be
   * misspelled, or `null` if nothing is misspelled.
   *
   * @param str - The string to decompose into spellchecked words.
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

  /**
   * Saves a word to the user's internal dictionary.
   *
   * @param word - The word to save. Capitalization is normalized and thus
   *   doesn't matter.
   */
  async saveToDictionary(word: string) {
    if (this.disabled) return
    word = lowercase(word, this.locale)
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
export const Spellchecker = new SpellcheckWorker(i18nLocale)
