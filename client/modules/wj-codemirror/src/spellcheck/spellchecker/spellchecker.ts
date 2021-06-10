import { SpellcheckerWasm, SuggestedItem } from "spellchecker-wasm/lib/browser/index"
import type { CheckSpellingOptions } from "spellchecker-wasm/lib/SpellCheckerBase"
import {
  createLock,
  isTitlecased,
  isUppercased,
  lowercase,
  titlecase,
  uppercase
} from "wj-util"
import locales from "./locales"
import type {
  Misspelling,
  SpellcheckerOptions,
  SpellcheckerURLS,
  Suggestion,
  Word
} from "./types"

// TODO: This entire strategy won't work for (afaik) Arabic and Chinese
// this is because we need to be able to split text into "words", which is much
// easier to do when writing with an alphabet. since segmenting text like that
// is not exactly easy, for the time being this spellchecker isn't very good
// for those kind of scripts.

const encoder = new TextEncoder()

const whitespace = (filtered: string) => " ".repeat(filtered.length)

export class Spellchecker {
  /** The internal `SpellcheckerWasm` instance that this class wraps around. */
  private declare readonly spellchecker: SpellcheckerWasm

  /**
   * The table of URLs pointing to the spellchecker's resources, such as
   * the dictionary.
   */
  private declare readonly urls: SpellcheckerURLS

  /** Spellchecker general configuration. */
  private declare readonly options: Required<SpellcheckerOptions>

  /** The spellchecker's locale, e.g. `"en"`. */
  declare readonly locale: string

  /** True if the spellchecker has been initialized. */
  declare ready: boolean

  /**
   * @param locale - The locale to set. This is used for normalizing strings.
   * @param urls - A table of URLs pointing to the spellchecker's
   *   resources, such as the dictionary.
   * @param options - Options to instantiate with.
   */
  constructor(locale: string, urls: SpellcheckerURLS, options?: SpellcheckerOptions) {
    const {
      compound = locales[locale]?.compound ?? false,
      distance = 2,
      unknown = true
    } = options ?? {}
    this.locale = locale
    this.urls = urls
    this.options = { compound, distance, unknown }
    this.spellchecker = new SpellcheckerWasm()
  }

  /** Starts the spellchecker, if it hasn't been already. */
  private async init() {
    if (this.ready) return
    const urls = this.urls
    const [wasm, dict, bigram] = urls.bigram
      ? await Promise.all([fetch(urls.wasm), fetch(urls.dict), fetch(urls.bigram)])
      : await Promise.all([fetch(urls.wasm), fetch(urls.dict)])
    await this.spellchecker.prepareSpellchecker(wasm, dict, bigram, {
      countThreshold: 4,
      dictionaryEditDistance: this.options.distance
    })
    this.ready = true
  }

  /**
   * Runs the spellchecker on a string.
   *
   * @param str - The string to spellcheck.
   * @param compound - If true, the string checked will be treated as a sentence.
   * @param opts - Options to run the spellchecker with.
   */
  private run = createLock(
    async (
      str: string,
      compound?: boolean,
      opts: CheckSpellingOptions = {
        maxEditDistance: this.options.distance,
        includeUnknown: this.options.unknown,
        includeSelf: true,
        verbosity: 1
      }
    ) => {
      if (!this.ready) await this.init()

      const lowered = lowercase(str, this.locale)

      const suggestions = await new Promise<SuggestedItem[]>(resolve => {
        // set handler to resolve our promise
        this.spellchecker.resultHandler = resolve
        // start the checker
        if (compound) this.spellchecker.checkSpellingCompound(lowered, opts)
        else this.spellchecker.checkSpelling(lowered, opts)
      })

      if (suggestions.length === 0) return null

      if (suggestions.length === 1 && suggestions[0].term === lowered) {
        // word is spelled correctly, but weird bug caused it to have a suggestion anyways
        if (suggestions[0].distance === 0) return null
        // word is misspelled, but no suggestions exist
        else return []
      }

      const out = this.processSuggestions(str, suggestions)
      return out.length ? out : null
    }
  )

  /**
   * Normalizes a string using a locale's configuration.
   *
   * @param str - The string to normalize.
   */
  private normalize(str: string) {
    let output = str

    const locale = locales[this.locale]

    if (locale?.replacements) {
      for (const [text, replacement] of locale.replacements) {
        output = output.replaceAll(text, replacement)
      }
    }

    if (locale?.filters) {
      for (const filter of locale.filters) {
        output = output.replaceAll(filter, whitespace)
      }
    }

    return output
  }

  /**
   * Splits a string into ranges of words.
   *
   * @param str - The string to find the words of.
   */
  private indexWords(str: string) {
    str = this.normalize(str)

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
    const titlecased = isTitlecased(word, this.locale)
    const uppercased = isUppercased(word, this.locale)

    // clamp maximum number of suggestions
    if (suggestions.length > 8) suggestions = suggestions.slice(0, 8)

    const out: Suggestion[] = []

    // convert to JSON, handle capitalization based on the original word
    for (const item of suggestions) {
      const suggestion = { ...item.toJSON() }

      if (titlecased || uppercased) {
        suggestion.term = uppercased
          ? uppercase(suggestion.term, this.locale)
          : titlecase(suggestion.term, this.locale)
      }

      out.push(suggestion)
    }

    return out
  }

  /**
   * Checks the spelling of a word. Returns `null` if nothing is misspelled.
   *
   * @param word - The word to spellcheck.
   */
  async check(word: string) {
    if (!this.ready) await this.init()
    return await this.run(word, false)
  }

  /**
   * Checks the spelling of a sentence. Attempts to segment the sentence
   * given into a reasonable set, which accounts compound words and other
   * forms of unsegmented text. Returns `null` if the sentence has nothing
   * to correct.
   *
   * @param sentence - The sentence to segment and then spellcheck.
   */
  async checkSentence(sentence: string) {
    if (!this.ready) await this.init()
    return await this.run(sentence, true)
  }

  /**
   * Finds the words of a string and then runs the spellchecker on each
   * one. Returns a list of words that the spellchecker believes to be
   * misspelled, or `null` if nothing is misspelled.
   *
   * @param str - The string to decompose into spellchecked words.
   */
  async checkWords(str: string) {
    const words = this.indexWords(str)
    if (!words) return null

    const out: Misspelling[] = []
    for (const word of words) {
      const suggestions = await this.run(word.word, this.options.compound)
      if (!suggestions) continue
      out.push({ ...word, suggestions })
    }

    return !out.length ? null : out
  }

  /**
   * Segments a given string of string of text. Simply returns the
   * spellchecker's best guess on how the string should be segmented, while
   * attempting to not change any of the spelling.
   *
   * @param str - The string to be segmented.
   */
  async segment(str: string) {
    if (!this.ready) await this.init()
    const suggestions = await this.run(str, true, {
      maxEditDistance: 0,
      includeUnknown: false,
      includeSelf: false,
      verbosity: 1
    })

    if (!suggestions || !suggestions.length) return str

    return suggestions[0].term
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
        ? lowercase(input, this.locale).replaceAll("\r\n", "\n")
        : input.map(word => `${lowercase(word, this.locale)} ${frequency}`).join("\n")

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
