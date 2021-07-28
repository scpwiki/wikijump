import { transfer, WorkerModule } from "threads-worker-module"
import { locale as i18nLocale, Pref } from "wj-state"
import { dedupe } from "wj-util"
import type { Word } from ".."
import DICTIONARIES from "../dicts"
import type { NSpellWorkerInterface } from "./nspell.worker"

/** Class for instantiating a web-workerized NSpell instance. */
export class NSpellWorker extends WorkerModule<NSpellWorkerInterface> {
  /** The current locale of the spellchecker. */
  declare locale: string

  /**
   * True if the spellchecker is disabled because a locale for which no
   * dictionary can be found was provided.
   */
  declare disabled: boolean

  /**
   * @param locale - The locale to use when finding dictionaries. Any
   *   locale can be provided, but only the language code will be used. A
   *   locale for which no dictionary can be found will simply disable the
   *   spellchecker.
   */
  constructor(locale = "en") {
    super("nspell", importWorker, {
      persist: true,
      init: async () => void (await this.set(locale, true))
    })
    this.disabled = false
    this.locale = localeLanguage(locale)
  }

  /**
   * @param locale - The locale to use when finding dictionaries. Any
   *   locale can be provided, but only the language code will be used. A
   *   locale for which no dictionary can be found will simply disable the
   *   spellchecker.
   * @param force - Forces the spellchecker to reset and download the
   *   locale dictionaries even if the locale provided is the same as the
   *   one the spellchecker already has loaded.
   */
  async set(locale: string, force = false) {
    locale = localeLanguage(locale)
    if (!force && this.locale === locale) return

    if (DICTIONARIES.hasOwnProperty(locale)) {
      const { aff, dic } = await DICTIONARIES[locale]()
      await this.invoke("set", aff, dic)

      // add local dictionary to spellchecker once it has started
      const localDictionary = this.getLocalDictionary()
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

  // -- DICTIONARY

  /**
   * Appends another dictionary to the spellchecker.
   *
   * @param url - An absolute URL to the dictionary (`.dic`).
   */
  async dictionary(url: string) {
    if (this.disabled) return
    await this.invoke("dictionary", url)
  }

  /**
   * Adds a personal dictionary to the spellchecker. Similar to appending a
   * normal dictionary, but words in the personal dictionary have some
   * preferential treatment.
   *
   * @param words - The word(s) to add.
   */
  async personal(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("personal", words)
  }

  /**
   * Adds word(s) to the spellchecker's in-memory dictionary.
   *
   * @param words - The word(s) to add.
   */
  async add(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("add", words)
  }

  /**
   * Removes words frmo the spellchecker's in-memory dictionary.
   *
   * @param words - The word(s) to remove.
   */
  async remove(words: string | string[]) {
    if (this.disabled) return
    await this.invoke("remove", words)
  }

  // -- SPELLCHECK

  /**
   * Determines if a word is spelled correctly.
   *
   * @param word - The word to check.
   */
  async correct(word: string) {
    if (this.disabled) return true
    return await this.invoke("correct", transfer(word))
  }

  /**
   * Returns an object containing metadata about a word.
   *
   * @param word - The word to check.
   */
  async info(word: string) {
    if (this.disabled) return { correct: true, forbidden: false, warn: false }
    return await this.invoke("info", transfer(word))
  }

  /**
   * Returns suggestions for how to correct the spelling of a word.
   *
   * @param word - The word to get the suggestions for.
   * @param max - The maximum number of suggestions.
   */
  async suggest(word: string, max = 8) {
    if (this.disabled) return []
    return await this.invoke("suggest", transfer(word), max)
  }

  /**
   * Takes in a list of misspelled words and returns them with suggestions
   * for correcting them.
   *
   * @param words - The words to get suggestions for.
   * @param max - The maximum number of suggestions.
   */
  async suggestions(words: Word[], max = 8) {
    if (this.disabled) return []
    return await this.invoke("suggestions", words, max)
  }

  /**
   * Takes in a list of words and returns the words that are misspelled.
   *
   * @param words - The words to check.
   */
  async misspelled(words: Word[]) {
    if (this.disabled) return []
    return await this.invoke("misspelled", words)
  }

  /**
   * Takes in a list of words and returns of which are flagged with being
   * misspelled, warned, or forbidden.
   *
   * @param words - The words to check.
   */
  async check(words: Word[]) {
    if (this.disabled) return []
    return await this.invoke("check", words)
  }

  // -- MISC

  /**
   * Get the extra word characters defined by the loaded affix file. Most
   * affix files don’t set these.
   */
  async wordCharacters() {
    if (this.disabled) return null
    return await this.invoke("wordCharacters")
  }

  // -- LOCAL DICTIONARY

  /**
   * Adds words to the user's local dictionary.
   *
   * @param words - The word(s) to add.
   */
  async addToLocalDictionary(words: string | string[]) {
    if (this.disabled) return
    if (typeof words === "string") words = [words]
    this.setLocalDictionary(dedupe(this.getLocalDictionary(), ...words))
    await this.invoke("add", words)
  }

  /**
   * Removes words from the user's local dictionary.
   *
   * @param words - The word(s) to remove.
   */
  async removeFromLocalDictionary(words: string | string[]) {
    if (this.disabled) return
    if (typeof words === "string") words = [words]
    this.setLocalDictionary(
      this.getLocalDictionary().filter(word => !words.includes(word))
    )
    await this.invoke("remove", words)
  }

  /** Returns the user's current local dictionary. */
  getLocalDictionary() {
    return Pref.get<string[]>("spellchecker-user-dictionary", [])
  }

  /** Directly sets what words are in the local dictionary. */
  private setLocalDictionary(words: string[]) {
    Pref.set("spellchecker-user-dictionary", words)
  }
}

export default new NSpellWorker(i18nLocale)

function localeLanguage(locale: string) {
  return locale.toLowerCase().split(/-|_/)[0]
}

async function importWorker() {
  return (await import("./nspell.worker?bundled-worker")).default
}
