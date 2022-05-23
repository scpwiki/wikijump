import { AbstractWorkerBase, Comlink } from "@wikijump/comlink"
import Locale from "@wikijump/fluent"
import { dedupe, LazySingleton, Pref } from "@wikijump/util"
import type { Espells } from "espells"
import DICTIONARIES from "../dicts"
import type { FlaggedWord, Word } from "../types"

const RemoteClassSingleton = new LazySingleton(() =>
  Comlink.wrap<typeof Espells>(
    new Worker(new URL("./worker", import.meta.url), { type: "module" })
  )
)

export class EspellsWorker extends AbstractWorkerBase.of<Espells>([
  "add",
  "addDictionary",
  "data",
  "lookup",
  "remove",
  "stems",
  "suggest"
]) {
  declare locale: string

  protected _baseDefaults = {
    add: undefined,
    remove: undefined,
    addDictionary: undefined,
    lookup: () => ({ correct: true, forbidden: false, warn: false }),
    data: () => new Map(),
    stems: () => [],
    suggest: () => []
  }

  constructor(locale = "en") {
    super()
    this.set(locale)
  }

  async set(locale: string) {
    this.locale = localeLanguage(locale)
    if (this.loaded) await this.start(true)
  }

  protected async _baseGetWorker() {
    if (DICTIONARIES.hasOwnProperty(this.locale)) {
      const { aff: affURL, dic: dicURL } = DICTIONARIES[this.locale]

      // we can't use the static fromURL method on Espells because of Comlink limitations
      // so it has to be reimplemented basically - we have to resolve the urls

      const aff = await fetch(affURL).then(res => res.text())
      const dic = Array.isArray(dicURL)
        ? await Promise.all(dicURL.map(url => fetch(url).then(res => res.text())))
        : await fetch(dicURL).then(res => res.text())

      const espells = await new (RemoteClassSingleton.get())({ aff, dic })

      // add local dictionary to spellchecker once it has started
      for (const word of this.getLocalDictionary()) {
        await espells.add(word)
      }

      return espells
    }

    return false
  }

  async check(words: Word[], caseSensitive?: boolean): Promise<FlaggedWord[]> {
    if (this.starting) await this.starting
    if (!this.worker) await this.start()

    const flagged: FlaggedWord[] = []
    for (const word of words) {
      const info = await this.lookup(word.word, caseSensitive)
      flagged.push({
        ...word,
        info
      })
    }

    return flagged.filter(
      ({ info: { correct, forbidden, warn } }) => !correct || forbidden || warn
    )
  }

  // -- LOCAL DICTIONARY

  /**
   * Adds words to the user's local dictionary.
   *
   * @param words - The word(s) to add.
   */
  async addToLocalDictionary(words: string | string[]) {
    if (!this.loaded) return
    if (typeof words === "string") words = [words]
    this.setLocalDictionary(dedupe(this.getLocalDictionary(), ...words))
    for (const word of words) {
      await this.worker!.add(word)
    }
  }

  /**
   * Removes words from the user's local dictionary.
   *
   * @param words - The word(s) to remove.
   */
  async removeFromLocalDictionary(words: string | string[]) {
    if (!this.loaded) return
    if (typeof words === "string") words = [words]
    this.setLocalDictionary(
      this.getLocalDictionary().filter(word => !words.includes(word))
    )
    for (const word of words) {
      await this.worker!.remove(word)
    }
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

export default new EspellsWorker(Locale.locale)

function localeLanguage(locale: string) {
  return locale.toLowerCase().split(/-|_/)[0]
}
