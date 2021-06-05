/** Patterns to filter out as "not words". */
const FILTERS = [
  /\p{Nd}+\p{L}+/gu, // e.g. 5GW, 20mm, etc.
  /\b's\b/gu, // replaces `'s` plurals because they otherwise create unknown words
  /\s'|'(?!\p{L})/gu, // replaces ' marks on the start or end of words, but not inside
  /\b[\p{Lu}\p{Nd}]{2,4}\b/gu // SCP, MTF, etc. capitalized acronyms/initialisms
]

/**
 * Pairs of strings, with the first string being text to replace with the
 * latter. Used for normalizing text.
 */
const REPLACEMENTS: [string, string][] = [
  ["’", "'"] // normalize right single quotation marks into apostrophes
]

/** Normalizes a string for the spellchecker. */
export function normalize(str: string) {
  let output = str
  for (const [text, replacement] of REPLACEMENTS) {
    output = output.replaceAll(text, replacement)
  }
  for (const filter of FILTERS) {
    output = output.replaceAll(filter, filtered => " ".repeat(filtered.length))
  }
  return output
}
