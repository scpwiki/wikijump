import { isTitlecased, isUppercased, lowercase, titlecase, uppercase } from "wj-util"
import initSymSpell, { SymSpell } from "../../../vendor/symspell"
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

let wasmStarted = false

const encoder = new TextEncoder()

const whitespace = (filtered: string) => " ".repeat(filtered.length)

export class Spellchecker {
  /** The internal `SpellcheckerWasm` instance that this class wraps around. */
  private declare spellchecker: SymSpell

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
      unknown = locales[locale]?.unknown ?? false,
      distance = 2
    } = options ?? {}
    this.locale = locale
    this.urls = urls
    this.options = { compound, distance, unknown }
  }

  /** Starts the spellchecker, if it hasn't been already. */
  private async init() {
    if (this.ready) return

    if (!wasmStarted) await initSymSpell(this.urls.wasm)

    wasmStarted = true

    this.spellchecker = new SymSpell({
      max_edit_distance: this.options.distance,
      prefix_length: 7,
      count_threshold: 1
    })

    this.spellchecker.load_dictionary(
      new Uint8Array(await (await fetch(this.urls.dict)).arrayBuffer()),
      {
        term_index: 0,
        count_index: 1,
        separator: " "
      }
    )

    if (this.urls.bigram) {
      this.spellchecker.load_bigram_dictionary(
        new Uint8Array(await (await fetch(this.urls.bigram)).arrayBuffer()),
        {
          term_index: 0,
          count_index: 2,
          separator: " "
        }
      )
    }

    this.ready = true
  }

  /**
   * Runs the spellchecker on a string.
   *
   * @param str - The string to spellcheck.
   * @param compound - If true, the string checked will be treated as a sentence.
   * @param opts - Options to run the spellchecker with.
   */
  private async run(
    str: string,
    compound?: boolean,
    opts: { distance: number; unknown: boolean } = {
      distance: this.options.distance,
      unknown: this.options.unknown
    }
  ) {
    if (!this.ready) await this.init()

    const normalized = lowercase(str, this.locale)

    const suggestions = compound
      ? this.spellchecker.lookup_compound(normalized, opts.distance)
      : this.spellchecker.lookup(normalized, 2, opts.distance)

    // word is misspelled, but no suggestions
    if (suggestions.length === 0) return opts.unknown ? [] : null

    // word is spelled correctly
    if (
      suggestions.length === 1 &&
      suggestions[0].distance === 0 &&
      suggestions[0].term === normalized
    ) {
      return null
    }

    const out = this.processSuggestions(str, suggestions)
    return out.length ? out : null
  }

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
  private async indexWords(str: string): Promise<Word[] | null> {
    str = this.normalize(str)

    // any span with no whitespace
    const matches = str.matchAll(/\S+/gu)
    if (!matches) return null

    const out: Word[] = []

    const push = (word: string, pos: number) => {
      out.push({ word, from: pos, to: pos + word.length })
    }

    for (const match of matches) {
      if (!match.index) continue
      if (this.options.compound) {
        const segmented = await this.segment(match[0])
        if (segmented === match[0]) {
          push(match[0], match.index)
        } else {
          let pos = 0
          for (const segment of segmented.split(/\s+/)) {
            push(segment, match.index + pos)
            pos += segment.length
          }
        }
      } else {
        push(match[0], match.index)
      }
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
  private processSuggestions(word: string, suggestions: Suggestion[]) {
    const titlecased = isTitlecased(word, this.locale)
    const uppercased = isUppercased(word, this.locale)

    // clamp maximum number of suggestions
    if (suggestions.length > 8) suggestions = suggestions.slice(0, 8)

    const out: Suggestion[] = []

    // convert to JSON, handle capitalization based on the original word
    for (const item of suggestions) {
      const suggestion = { ...item }

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
   * Destroys the spellchecker's internal WASM instance, freeing memory.
   * This doesn't render the spellchecker inoperable, instead, the
   * spellchecker will restart itself if called again.
   */
  free() {
    if (!this.ready) return
    try {
      this.spellchecker.free()
    } finally {
      // @ts-ignore
      this.spellchecker = undefined
      this.ready = false
    }
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
    const words = await this.indexWords(str)
    if (!words) return null

    const out: Misspelling[] = []
    for (const word of words) {
      let suggestions = await this.run(word.word)

      // if no suggstions for misspelled, and compound, try again using a compound search
      if (suggestions && !suggestions.length && this.options.compound) {
        suggestions = await this.run(word.word, true)
        // if we have suggestions, add a suggestion for a compounded version
        if (suggestions) {
          suggestions = suggestions.flatMap(({ count, distance, term }) => [
            {
              count,
              distance,
              term: term.replaceAll(" ", "")
            },
            {
              count,
              distance,
              term
            }
          ])
        }
      }

      if (suggestions) out.push({ ...word, suggestions })
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
      distance: 0,
      unknown: false
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

    this.spellchecker.load_dictionary(encoder.encode(dict), {
      term_index: 0,
      count_index: 1,
      separator: " "
    })
  }
}
