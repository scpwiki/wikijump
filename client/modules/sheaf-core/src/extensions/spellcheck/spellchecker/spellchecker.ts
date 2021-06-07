import { SpellcheckerWasm, SuggestedItem } from "spellchecker-wasm/lib/browser/index"
import { capitalize, isCapitalized, isUppercased } from "wj-util"
import { normalize } from "./normalize"

// TODO: This entire strategy won't work for (afaik) Arabic and Chinese
// this is because we need to be able to split text into "words", which is much
// easier to do when writing with an alphabet. since segmenting text like that
// is not exactly easy, for the time being this spellchecker isn't very good
// for those kind of scripts.

/** A suggestion for replacing a word. */
export type Suggestion = {
  /** The frequency of the suggested word within the corpus. */
  count: number
  /** The distance from misspelled word to the suggested word. */
  distance: number
  /** The suggested word. */
  term: string
}

/** Describes a word in a document, i.e. the term itself and its location. */
export interface Word {
  /** The word itself in the document. */
  word: string
  /** The starting position of the word in the document. */
  from: number
  /** The ending position of the word in the document. */
  to: number
}

/** Describes a misspelled word along with suggestions for correcting it. */
export interface Misspelling extends Word {
  /** A list of suggestions for correcting the misspelling. */
  suggestions: Suggestion[]
}

/** A table of URLs pointing to a spellchecker's resources, such as its dictionary. */
export interface SpellcheckerURLS {
  /** An absolute URL to the `spellchecker-wasm` WASM binary. */
  wasm: string
  /** An absolute URL to a FrequencyWords dictionary text file. */
  dict: string
  /** An absolute URL to a bigram text file, if available. */
  bigram?: string
}

const encoder = new TextEncoder()

export class Spellchecker {
  /** The internal `SpellcheckerWasm` instance that this class wraps around. */
  private declare readonly spellchecker: SpellcheckerWasm

  /**
   * The table of URLs pointing to the spellchecker's resources, such as
   * the dictionary.
   */
  private declare readonly urls: SpellcheckerURLS

  /** The spellchecker's locale, e.g. `"en"`. */
  declare readonly locale: string

  /**
   * True if the spellchecker is using a bigram dictionary, which lets it
   * split compound words.
   */
  declare readonly usingBigrams: boolean

  /** True if the spellchecker has been initialized. */
  declare ready: boolean

  /**
   * @param locale - The locale to set. This is used for normalizing strings.
   * @param urls - A table of URLs pointing to the spellchecker's
   *   resources, such as the dictionary.
   */
  constructor(locale: string, urls: SpellcheckerURLS) {
    this.locale = locale
    this.urls = urls
    this.usingBigrams = Boolean(urls.bigram)
    this.spellchecker = new SpellcheckerWasm()
  }

  private async init() {
    if (this.ready) return
    const urls = this.urls
    const [wasm, dict, bigram] = urls.bigram
      ? await Promise.all([fetch(urls.wasm), fetch(urls.dict), fetch(urls.bigram)])
      : await Promise.all([fetch(urls.wasm), fetch(urls.dict)])
    await this.spellchecker.prepareSpellchecker(wasm, dict, bigram)
    this.ready = true
  }

  /**
   * Splits a string into ranges of words.
   *
   * @param str - The string to find the words of.
   */
  private indexWords(str: string) {
    str = normalize(str, this.locale)

    // any span with no whitespace
    const matches = str.matchAll(/\S+/gu)

    if (!matches) return null
    const out: Word[] = []

    for (const match of matches) {
      if (!match.index) continue
      out.push({ word: match[0], from: match.index, to: match.index + match[0].length })
    }

    return !out.length ? null : out
  }

  /**
   * Processes a list of suggestions for a misspelled word. Specifically,
   * the suggestion list is truncated to 8 items, and the capitalization of
   * the suggestions is changed to match the original misspelled word.
   *
   * @param word - The misspelled word.
   * @param suggestions - The suggestions provided by the spellchecker for this word.
   */
  private processSuggestions(word: string, suggestions: SuggestedItem[]) {
    const capitalized = isCapitalized(word)
    const uppercased = isUppercased(word)

    // clamp maximum number of suggestions
    if (suggestions.length > 8) suggestions = suggestions.slice(0, 8)

    const out: Suggestion[] = []

    // convert to JSON, handle capitalization based on the original word
    for (const item of suggestions) {
      const suggestion = { ...item.toJSON() }

      if (capitalized || uppercased) {
        suggestion.term = uppercased
          ? suggestion.term.toUpperCase()
          : capitalize(suggestion.term)
      }

      out.push(suggestion)
    }

    return out
  }

  /**
   * Checks the spelling of a word.
   *
   * @param word - The word to spellcheck.
   */
  async check(word: string) {
    if (!this.ready) await this.init()
    const suggestions = await new Promise<Suggestion[]>(resolve => {
      // set handler to resolve our promise
      this.spellchecker.resultHandler = items =>
        resolve(this.processSuggestions(word, items))

      // start the checker
      if (this.usingBigrams) {
        this.spellchecker.checkSpellingCompound(word.toLowerCase())
      } else {
        this.spellchecker.checkSpelling(word.toLowerCase())
      }
    })

    return suggestions.length ? suggestions : null
  }

  /**
   * Finds the words of a string and then runs the spellchecker on each
   * one. Returns a list of words that the spellchecker believes to be misspelled.
   */
  async checkWords(str: string) {
    const words = this.indexWords(str)
    if (!words) return null

    const out: Misspelling[] = []
    for (const word of words) {
      const suggestions = await this.check(word.word)
      if (!suggestions) continue
      out.push({ ...word, suggestions })
    }

    return !out.length ? null : out
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
    if (!this.ready) await this.init()

    const dict =
      typeof input === "string"
        ? input.toLowerCase().replaceAll("\r\n", "\n")
        : input.map(word => `${word.toLowerCase()} ${frequency}`).join("\n")

    // spellchecker-wasm recommends streaming in chunks at 32kb-64kb.
    // so we'll split into lines, and then turn that array into a bunch of
    // 32kb-ish chunks. this may be overkill for adding a few words, but
    // if adding a large dictionary it's needed.

    const lines = dict.split("\n")
    const chunks: string[] = ["\n"]

    while (lines.length) {
      const last = chunks[chunks.length - 1]
      if (last.length >= 32768) {
        chunks.push("")
        continue
      }
      chunks[chunks.length - 1] = `${last + lines.shift()!}\n`
    }

    for (const chunk of chunks) {
      const encoded = encoder.encode(chunk)
      this.spellchecker.writeToDictionary(encoded)
    }
  }
}
