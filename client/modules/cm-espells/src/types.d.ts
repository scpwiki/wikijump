import type { Tree } from "@lezer/common"
import type { EditorState } from "@wikijump/codemirror/cm"

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
  suggestions: string[]
}

/**
 * {@link Word} flagged by the spellchecker as being "incorrect" in some
 * way. This may be due to the word being misspelled, forbidden, and/or warned.
 */
export interface FlaggedWord extends Word {
  /** Metadata about the spellcheck status of the word. */
  info: {
    /** True if the word isn't misspelled. */
    correct: boolean
    /** True if the spellcheck dictionary forbids this word. */
    forbidden: boolean
    /**
     * True if the spellcheck dictionary notes this word as being spelled
     * correctly, but is regardless most likely an error.
     */
    warn: boolean
  }
}

/**
 * Describes an asynchronous function that returns absolute URLs to a
 * Hunspell `.aff` and `.dic` pair.
 */
export type DictionaryImporter = () => Promise<{
  /** Absolute URL to the affix file to be loaded. */
  aff: string
  /** Absolute URL(s) to the dictionary file(s) to be loaded. */
  dic: string | string[]
}>

/**
 * A function provided by a language's `languageData` `spellcheck`
 * property. Determines if a given word should be spellchecked or not.
 *
 * @param state - The editor state when the word was found.
 * @param tree - The current syntax tree.
 * @param word - The word to potentially be filtered.
 */
export type SpellcheckFilter = (state: EditorState, tree: Tree, word: Word) => boolean

/**
 * A function that can be provided by a locale's `filter` list. If `true`
 * is returned, the word being checked will be excluded.
 *
 * @param word - The word to potentially be filtered.
 */
export type LocaleFilterFunction = (word: string) => boolean

/**
 * Describes a special configuration for a locale, such as how it matches
 * words or what words it excludes from spellchecking.
 */
export interface Locale {
  /** A global `RegExp` that determines what parts of a string are "words". */
  pattern: RegExp
  /**
   * A list of `RegExp`s or {@link LocaleFilterFunction} functions that, if
   * they test/return true, exclude a word from being spellchecked.
   */
  filters: (RegExp | LocaleFilterFunction)[]
}
