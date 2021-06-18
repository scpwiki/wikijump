import iterate from "iterare"
import { Aff } from "./aff"
import { Dic } from "./dic"
import { Word } from "./dic/word"
import { Lookup } from "./lookup"
import { Reader } from "./reader"
import { Suggest } from "./suggest"

export interface EspellsInitOpts {
  /** Source of a `.aff` affix file. */
  aff: string | Uint8Array
  /** A source for a single or array of `.dic` files. */
  dic: string | Uint8Array | (string | Uint8Array)[]
}

const decoder = new TextDecoder()

/**
 * Espells spellchecker. Instances of this class are fully instantiated
 * when created and do not need any special init functions called.
 */
export class Espells {
  private declare aff: Aff
  private declare dic: Dic
  private declare lookuper: Lookup
  private declare suggester: Suggest

  constructor({ aff, dic }: EspellsInitOpts) {
    // concatenate every dictionary
    // TODO: is the sorting an issue?
    if (!Array.isArray(dic)) dic = [dic]
    dic = dic.reduce(
      (acc, cur) => acc + (typeof cur === "string" ? cur : decoder.decode(cur)),
      ""
    )

    this.aff = new Aff(new Reader(aff))
    this.dic = new Dic(new Reader(dic), this.aff)
    this.lookuper = new Lookup(this.aff, this.dic)
    this.suggester = new Suggest(this.aff, this.dic, this.lookuper)
  }

  /**
   * Creates an {@link ESpells} instance from URL strings rather than raw
   * sources. Uses fetch, assumes the given URL can be decoded into a
   * string of text.
   */
  static async fromURL(opts: { aff: string; dic: string | string[] } & EspellsInitOpts) {
    const aff = await (await fetch(opts.aff)).text()

    const dicURLs = typeof opts.dic === "string" ? [opts.dic] : opts.dic
    const dic: string[] = []
    for (const url of dicURLs) {
      const data = await (await fetch(url)).text()
      dic.push(data)
    }

    return new Espells({ ...opts, aff, dic })
  }

  /**
   * Adds a dictionary (or array of dictionaries) to the current instance.
   *
   * @param dic - The dictionary (or dictionaries) to add.
   */
  addDictionary(dic: string | Uint8Array | (string | Uint8Array)[]) {
    if (!Array.isArray(dic)) dic = [dic]
    for (const input of dic) {
      this.dic.addDictionary(new Reader(input))
    }
  }

  /** Adds a word to the spellchecker's dictionary. */
  add(stem: string) {
    const word = new Word(stem, this.aff)
    this.dic.add(word)
  }

  /** Removes a word from the spellchecker's dictionary. */
  remove(stem: string) {
    this.dic.remove(stem)
  }

  /**
   * Determines if a word is spelled correctly (or at least, can be found
   * in the dictionary).
   *
   * @param word - The word to check.
   * @param caseSensitive - If true, the spellchecker will consider the
   *   capitalization of the word given. Defaults to true.
   */
  lookup(word: string, caseSensitive = true) {
    return this.lookuper.test(word, caseSensitive)
  }

  suggest(word: string, max = 8) {
    return iterate(this.suggester.suggestions(word))
      .take(max)
      .map(suggestion => suggestion.text)
      .toArray()
  }
}
