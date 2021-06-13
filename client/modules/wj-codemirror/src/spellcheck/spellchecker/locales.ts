import type { SpellcheckerLocale } from "./types"

// prettier-ignore
export const LOCALES: Record<string, SpellcheckerLocale> = {
  "en": {
    replacements: [
      ["’", "'"] // normalize right single quotation marks into apostrophes
    ],
    filters: [
      /[^\p{L}]\p{L}(?![\p{L}'])/gu, // single characters
      /\p{Nd}+\p{L}+/gu,             // e.g. 5GW, 20mm, etc.
      /\b's\b/gu,                    // remove `'s` plurals
      /\s'|'(?!\p{L})/gu,            // remove ' marks on the start/end of words
      /\b[\p{Lu}\p{Nd}]{2,4}\b/gu,   // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}'\s]+/gu               // filter punctuation out
    ],
    unknown: true
  },

  "de": {
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,  // single characters
      /\p{Nd}+\p{L}+/gu,           // e.g. 5GW, 20mm, etc.
      /\b[\p{Lu}\p{Nd}]{2,4}\b/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu              // filter punctuation out
    ],
    compound: true
  },

  "es":{
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,  // single characters
      /\p{Nd}+\p{L}+/gu,           // e.g. 5GW, 20mm, etc.
      /\b[\p{Lu}\p{Nd}]{2,4}\b/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu              // filter punctuation out
    ]
  },

  "fr": {
    filters: [
      /[^\p{L}\s]+/gu                           // filter punctuation out
    ]
  },

  "it": {
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,  // single characters
      /\p{Nd}+\p{L}+/gu,           // e.g. 5GW, 20mm, etc.
      /\b[\p{Lu}\p{Nd}]{2,4}\b/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu              // filter punctuation out
    ]
  },

  "ru": {
    filters: [
      /[^\p{L}\s-]+/gu                           // filter punctuation out
    ],
    compound: true
  }
}

export default LOCALES
