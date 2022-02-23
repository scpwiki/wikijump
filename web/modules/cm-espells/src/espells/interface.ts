import { bindMethods, Comlink, type Remote, type RemoteObject } from "@wikijump/comlink"
import Locale from "@wikijump/fluent"
import { dedupe, Pref } from "@wikijump/util"
import type { Espells } from "espells"
import DICTIONARIES from "../dicts"
import type { FlaggedWord, Word } from "../types"

type RemoteEspells = Remote<Espells>

export class EspellsWorker implements RemoteObject<Espells> {
  declare locale: string

  declare disabled: boolean

  declare loading: Promise<void>

  private declare worker: RemoteEspells

  declare add: RemoteEspells["add"]
  declare addDictionary: RemoteEspells["addDictionary"]
  declare data: RemoteEspells["data"]
  declare lookup: RemoteEspells["lookup"]
  declare remove: RemoteEspells["remove"]
  declare stems: RemoteEspells["stems"]
  declare suggest: RemoteEspells["suggest"]

  constructor(locale = "en") {
    this.set(locale)
  }

  async set(locale: string) {
    if (this.loading) await this.loading
    this.locale = localeLanguage(locale)
    this.loading = this.init()
    await this.loading
  }

  private async init() {
    if (DICTIONARIES.hasOwnProperty(this.locale)) {
      const workerClass = Comlink.wrap<typeof Espells>(
        new (await import("./worker?worker")).default()
      )

      const { aff: affURL, dic: dicURL } = await DICTIONARIES[this.locale]()

      // we can't use the static fromURL method on Espells because of Comlink limitations
      // so it has to be reimplemented basically - we have to resolve the urls

      const aff = await fetch(affURL).then(res => res.text())
      const dic = Array.isArray(dicURL)
        ? await Promise.all(dicURL.map(dic => fetch(dic))).then(res =>
            Promise.all(res.map(res => res.text()))
          )
        : await fetch(dicURL).then(res => res.text())

      this.worker = await new workerClass({ aff, dic })

      bindMethods({
        target: this,
        worker: this.worker,
        methods: ["add", "addDictionary", "data", "lookup", "remove", "stems", "suggest"],
        check: async () => {
          await this.loading
          if (this.disabled) throw new Error("No worker!")
        }
      })

      // add local dictionary to spellchecker once it has started
      for (const word of this.getLocalDictionary()) {
        await this.worker.add(word)
      }
    } else {
      console.warn("Locale given to spellchecker has no resources available for it.")
      this.disabled = true
    }
  }

  async check(words: Word[], caseSensitive?: boolean): Promise<FlaggedWord[]> {
    await this.loading
    const flagged: FlaggedWord[] = []
    for (const word of words) {
      const info = await this.worker.lookup(word.word, caseSensitive)
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
    if (this.disabled) return
    if (typeof words === "string") words = [words]
    this.setLocalDictionary(dedupe(this.getLocalDictionary(), ...words))
    for (const word of words) {
      await this.worker.add(word)
    }
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
    for (const word of words) {
      await this.worker.remove(word)
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
