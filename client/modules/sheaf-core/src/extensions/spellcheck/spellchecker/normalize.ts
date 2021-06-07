/**
 * Pairs of strings, with the first string being text to replace with the
 * latter. Used for normalizing text.
 */
const REPLACEMENTS: [string, string][] = [
  ["’", "'"] // normalize right single quotation marks into apostrophes
]

/** Returns a list of patterns to filter out as "not words", depending on locale. */
function getFilters(locale: string): RegExp[] {
  switch (locale) {
    case "en":
      return [
        /\p{Nd}+\p{L}+/gu, // e.g. 5GW, 20mm, etc.
        /\b's\b/gu, // replaces `'s` plurals because they otherwise create unknown words
        /\s'|'(?!\p{L})/gu, // replaces ' marks on the start/end of words, but not inside
        /\b[\p{Lu}\p{Nd}]{2,4}\b/gu // SCP, MTF, etc. capitalized initialisms
      ]

    default:
      return []
  }
}

/** Normalizes a string for the spellchecker. */
export function normalize(str: string, locale: string) {
  let output = str

  for (const [text, replacement] of REPLACEMENTS) {
    output = output.replaceAll(text, replacement)
  }

  const filters = getFilters(locale)
  for (const filter of filters) {
    output = output.replaceAll(filter, filtered => " ".repeat(filtered.length))
  }

  // get rid of all punctuation except stuff that goes into words (or whitespace)
  output = output.replaceAll(/[^\p{L}'\s]+/gu, match => " ".repeat(match.length))

  return output
}
