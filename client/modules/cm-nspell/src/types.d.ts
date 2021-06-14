import type { Tree } from "lezer-tree"
import type { EditorState } from "wj-codemirror/cm"

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
