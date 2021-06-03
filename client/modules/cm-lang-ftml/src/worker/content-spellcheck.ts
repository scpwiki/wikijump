import { SpellcheckerWasm, SuggestedItem } from "spellchecker-wasm/lib/browser/index"
import { capitalize, isCapitalized, isUppercased } from "wj-util"

let spellchecker: SpellcheckerWasm | null = null
let usingBigrams = false

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

/**
 * Patterns to filter out as "not words". Used by {@link checkWords} and
 * {@link indexWords}.
 */
const FILTERS = [
  /\p{Nd}+\p{L}+/gu, // e.g. 5GW, 20mm, etc.
  /\b's\b/gu, // replaces `'s` plurals because they otherwise create unknown words
  /\s'|'\s/gu, // replaces ' marks on the start or end of words, but not inside
  /\b[\p{Lu}\p{Nd}]{2,4}\b/gu // SCP, MTF, etc. capitalized acronyms/initialisms
]

/**
 * Pairs of strings, with the first string being text to replace with the
 * latter. Used for normalizing text.
 */
const REPLACEMENTS: [string, string][] = [
  ["’", "'"] // normalize right single quotation marks into apostrophes
]

function normalize(str: string) {
  let output = str
  for (const [text, replacement] of REPLACEMENTS) {
    output = output.replaceAll(text, replacement)
  }
  for (const filter of FILTERS) {
    output = output.replaceAll(filter, filtered => " ".repeat(filtered.length))
  }
  return output
}

/**
 * Sets the spellchecker configuration. No other spellchecking functions
 * will work until this is done at least once. Calling this function more
 * than once is fine, and is in-fact how you reconfigure the spellchecker.
 *
 * @param wasmURL - An absolute URL to the `spellchecker-wasm` WASM binary.
 * @param dict - An absolute URL to a FrequencyWords dictionary text file.
 * @param bigramURL - An absolute URL to a bigram text file, if desired.
 */
export async function setSpellchecker(
  wasmURL: string,
  dictURL: string,
  bigramURL?: string
) {
  if (bigramURL) usingBigrams = true
  const [wasm, dict, bigram] = bigramURL
    ? await Promise.all([fetch(wasmURL), fetch(dictURL), fetch(bigramURL)])
    : await Promise.all([fetch(wasmURL), fetch(dictURL)])

  spellchecker = new SpellcheckerWasm()
  await spellchecker.prepareSpellchecker(wasm, dict, bigram)
}

/**
 * Splits a string into ranges of words. The algorithm used is simple, so
 * it's recommended that the string given be relatively simple.
 *
 * @param str - The string to find the words of.
 */
export function indexWords(str: string) {
  // get rid of all punctuation except stuff that goes into words (or whitespace)
  str = str.replaceAll(/[^\p{L}'\s]+/gu, match => " ".repeat(match.length))

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

function processSuggestions(word: string, suggestions: SuggestedItem[]) {
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
export function check(word: string) {
  if (!spellchecker) throw new Error("Spellchecker wasn't started first!")
  return new Promise<Suggestion[]>(resolve => {
    spellchecker!.resultHandler = items => resolve(processSuggestions(word, items))
    if (usingBigrams) spellchecker!.checkSpellingCompound(word.toLowerCase())
    else spellchecker!.checkSpelling(word.toLowerCase())
  })
}

/**
 * Finds the words of a string using {@link indexWords} and then runs the
 * spellchecker on each one. Returns a list of words that the spellchecker
 * believes to be misspelled.
 */
export async function checkWords(str: string) {
  if (!spellchecker) throw new Error("Spellchecker wasn't started first!")

  const words = indexWords(normalize(str))
  if (!words) return null

  const out: Misspelling[] = []
  for (const word of words) {
    const suggestions = await check(word.word)
    if (!suggestions.length) continue
    out.push({ ...word, suggestions })
  }

  return !out.length ? null : out
}

const encoder = new TextEncoder()

/**
 * Appends a dictionary (a string) or a list of words to the current
 * spellchecker's dictionary. If directly appending a dictionary, it should
 * be noted that it is in a word frequency format.
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
export function appendToDictionary(input: string | string[], frequency = 1000) {
  if (!spellchecker) throw new Error("Spellchecker wasn't started first!")

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
    spellchecker.writeToDictionary(encoded)
  }
}
