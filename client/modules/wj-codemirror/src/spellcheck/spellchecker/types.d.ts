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

export interface SpellcheckerOptions {
  /**
   * If true, found "words" will instead be treated as sentences. This is
   * useful for scripts that do not separate words with a punctuation or
   * whitespace character.
   */
  compound?: boolean
  /**
   * The maximum "distance" between a string and a word in the dictionary,
   * comparatively. A higher distance value reduces the threshold for
   * "similarity" between a misspelled word and a potential suggestion.
   */
  distance?: number
  /**
   * If true, the spellchecker will mark misspelled words even if it has no
   * suggestions available.
   */
  unknown?: boolean
}

export interface SpellcheckerLocale {
  /**
   * Pairs of strings, with the first string being text to replace with the
   * latter. Used for normalizing text. Both strings need to be the same length.
   */
  replacements?: [text: string, replacement: string][]
  /** Returns a list of patterns to filter out as "not words". */
  filters?: RegExp[]
  /**
   * If true, the language will have "words" found treated as sentences,
   * i.e. they're segmented first when checked.
   */
  compound?: boolean
}

/** A table of URLs describing where to retrieve a frequency dictionary for a locale. */
export interface Dictionary {
  /** A URL to the frequency dictionary for a locale. */
  dict: string
  /** An optional URL to a bigram frequency dictionary for a locale. */
  bigram?: string
}
