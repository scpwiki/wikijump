import iterate from "iterare"
import { Aff } from "./aff"
import { decoder } from "./constants"
import { Dic } from "./dic"
import { Word } from "./dic/word"
import { Lookup } from "./lookup"
import { Reader } from "./reader"
import { Suggest } from "./suggest"
import { concat } from "./util"

export type {
  AffData,
  CharacterMap,
  Flag,
  Flags,
  FlagSet,
  PrefixIndex,
  PrefixMap,
  SuffixIndex,
  SuffixMap
} from "./aff"
export type { LookupResult } from "./lookup"

export interface EspellsInitOpts {
  /** Source of a `.aff` affix file. */
  aff: string | Uint8Array
  /** A source for a single or array of `.dic` files. */
  dic: string | Uint8Array | (string | Uint8Array)[]
}

/**
 * Espells spellchecker. Instances of this class are fully instantiated
 * when created and do not need any special init functions called.
 */
export class Espells {
  /** The {@link Aff} instance for the spellchecker. */
  private declare aff: Aff

  /** The {@link Dic} instance for the spellchecker. */
  private declare dic: Dic

  /** The {@link Lookup} instance for the spellchecker. */
  private declare lookuper: Lookup

  /** The {@link Suggest} instance for the spellchecker. */
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
   * Determines if a word meets three different criteria:
   *
   * - If the word is spelled correctly
   * - If the word has been marked as forbidden
   * - If the word has been marked as `WARN`
   *
   * These are the `correct`, `forbidden`, and `warn` properties of the
   * returned object, respectively.
   *
   * @param word - The word to check.
   * @param caseSensitive - If true, the spellchecker will consider the
   *   capitalization of the word given. Defaults to true.
   */
  lookup(word: string, caseSensitive = true) {
    return this.lookuper.check(word, caseSensitive)
  }

  /**
   * Returns suggestions for a word, even if it isn't misspelled.
   *
   * @param word - The word to get the suggestions of.
   * @param max - The maximum number of suggestions to return. Defaults to 8.
   */
  suggest(word: string, max = 8) {
    return iterate(this.suggester.suggestions(word))
      .take(max)
      .map(suggestion => suggestion.text)
      .toArray()
  }

  /**
   * Returns the stems for a word, which are all of the potential "base
   * forms" of a word, which will have various suffixes or prefixes
   * attached to that base to make the given word.
   *
   * If the word given is misspelled, the array of stems returned will just be empty.
   *
   * @param word - The word to get the stems of.
   * @param caseSensitive - If true, the spellchecker will consider the
   *   capitalization of the word given. Defaults to true.
   */
  stems(word: string, caseSensitive = true) {
    if (!this.lookup(word).correct) return []
    return iterate(this.lookuper.stems(word, { caps: caseSensitive })).toArray()
  }

  /**
   * Returns the "morphological data" for a word. This data is basically
   * just a map of keys and values, representing some sort of metadata
   * attached to a stem. e.g. a potential key-value could be `is:gendered`
   * (for some languages), which could be checked like:
   *
   * ```ts
   * const gendered =
   *   espells.data("word").get("is")?.has("gendered") ?? false
   * ```
   *
   * The reason for there being a `Set` assigned to a key is because you
   * could have multiple values under the `"is"` key, like `is:x`, `is:y`, etc.
   *
   * It should also be noted that this function takes care to get every
   * stem of the word, and then merge the morphological data for every
   * stem, which is what is finally returned. If you want to take more care
   * than that, and get only the data attached to a specific stem, you
   * could use the {@link Espells.stems} function first, and use one of the
   * stems it returns.
   *
   * The last detail to mention is that if the word is misspelled, the map
   * returned will just be empty.
   *
   * @param word - The word to get the data of.
   * @param caseSensitive - If true, the spellchecker will consider the
   *   capitalization of the word given. Defaults to true.
   */
  data(word: string, caseSensitive = true) {
    // process:
    // * check if the word is correct
    // * get the stems of the word
    // * get the data maps for every homonym of the stems
    // * reduce every map into a single, merged map without overwriting anything

    if (!this.lookup(word).correct) return new Map<string, Set<string>>()

    return iterate(this.lookuper.stems(word, { caps: caseSensitive }))
      .map(stem => this.lookuper.data(stem, caseSensitive))
      .reduce((acc, cur) => {
        iterate(cur)
          .flatten()
          .forEach(([key, set]) => {
            acc.set(key, concat(acc.get(key) ?? new Set<string>(), set))
          })
        return acc
      }, new Map<string, Set<string>>())
  }
}
