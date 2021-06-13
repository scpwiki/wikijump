import type { SpellcheckerLocale } from "./types"

// prettier-ignore
export const LOCALES: Record<string, SpellcheckerLocale> = {
  "en": {
    replacements: [
      ["’", "'"] // normalize right single quotation marks into apostrophes
    ],
    filters: [
      /[^\p{L}']\p{L}(?![\p{L}'])/gu,           // single characters
      /\p{Nd}+\p{L}+/gu,                        // e.g. 5GW, 20mm, etc.
      /\b's\b/gu,                               // remove `'s` plurals
      /[^\p{L}]'|'(?!\p{L})/gu,                 // remove ' marks on start/end of words
      /[^\p{L}][\p{Lu}\p{Nd}]{2,5}(?!\p{L})/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}'\s]+/gu                          // filter punctuation out
    ],
    unknown: true
  },

  "de": {
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,               // single characters
      /\p{Nd}+\p{L}+/gu,                        // e.g. 5GW, 20mm, etc.
      /[^\p{L}][\p{Lu}\p{Nd}]{2,5}(?!\p{L})/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu                           // filter punctuation out
    ],
    compound: true
  },

  "es":{
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,               // single characters
      /\p{Nd}+\p{L}+/gu,                        // e.g. 5GW, 20mm, etc.
      /[^\p{L}][\p{Lu}\p{Nd}]{2,5}(?!\p{L})/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu                           // filter punctuation out
    ],
    unknown: true
  },

  "fr": {
    filters: [
      /[^\p{L}]\p{L}(?!\p{L})/gu,               // single characters
      /\p{Nd}+\p{L}+/gu,                        // e.g. 5GW, 20mm, etc.
      /[^\p{L}][\p{Lu}\p{Nd}]{2,5}(?!\p{L})/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}\s]+/gu                           // filter punctuation out
    ]
  },

  "it": {
    filters: [
      /[^\p{L}']\p{L}(?![\p{L}'])/gu,           // single characters
      /[^\p{L}]\p{L}+'(?=\p{L})/gu,             // elision
      /[^\p{L}]'|'(?!\p{L})/gu,                 // remove ' marks on start/end of words
      /\p{Nd}+\p{L}+/gu,                        // e.g. 5GW, 20mm, etc.
      /[^\p{L}][\p{Lu}\p{Nd}]{2,5}(?!\p{L})/gu, // SCP, MTF, etc. capitalized initialisms
      /[^\p{L}'\s]+/gu                          // filter punctuation out
    ],
    unknown: true
  },

  // TODO: handle stress accents? e.g. ё vs е
  "ru": {
    filters: [
      /[^\p{L}-]\p{L}(?![\p{L}-])/gu,            // single characters
      /\p{Nd}+\p{L}+/gu,                         // e.g. 5GW, 20mm, etc.
      /[^\p{L}][\p{Lu}\p{Nd}-]{2,5}(?!\p{L})/gu, // capitalized initialisms
      /[^\p{L}\s-]+/gu                           // filter punctuation out
    ],
    unknown: true
  }
}

export default LOCALES
