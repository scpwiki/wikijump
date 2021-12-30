import { url } from "@wikijump/util"
import type { DictionaryImporter } from "./types"

/** Table of dictionary import functions, using locale language codes as keys. */
export const DICTIONARIES: Record<string, DictionaryImporter> = {
  "en": async () =>
    imp(import("dictionary-en/index.aff"), import("dictionary-en/index.dic")),

  "de": async () =>
    imp(import("../vendor/de.aff"), [
      import("../vendor/de-transam.dic"),
      import("../vendor/de-bjoern.dic"),
      import("../vendor/de-chrome.dic")
    ]),

  "es": async () =>
    imp(import("dictionary-es/index.aff"), import("dictionary-es/index.dic")),

  "fr": async () =>
    imp(import("dictionary-fr/index.aff"), import("dictionary-fr/index.dic")),

  "it": async () =>
    imp(import("dictionary-it/index.aff"), import("dictionary-it/index.dic")),

  "ru": async () =>
    imp(import("dictionary-ru/index.aff"), import("dictionary-ru/index.dic")),

  "ko": async () =>
    imp(import("dictionary-ko/index.aff"), import("dictionary-ko/index.dic")),

  "pl": async () =>
    imp(import("dictionary-pl/index.aff"), import("dictionary-pl/index.dic")),

  "uk": async () =>
    imp(import("dictionary-uk/index.aff"), import("dictionary-uk/index.dic")),

  "pt": async () =>
    imp(import("dictionary-pt/index.aff"), import("dictionary-pt/index.dic")),

  "cs": async () =>
    imp(import("dictionary-cs/index.aff"), import("dictionary-cs/index.dic")),

  "vi": async () =>
    imp(import("dictionary-vi/index.aff"), import("dictionary-vi/index.dic")),

  "el": async () =>
    imp(import("dictionary-el/index.aff"), import("dictionary-el/index.dic")),

  "tr": async () =>
    imp(import("dictionary-tr/index.aff"), import("dictionary-tr/index.dic")),

  "da": async () =>
    imp(import("dictionary-da/index.aff"), import("dictionary-da/index.dic")),

  "nb": async () =>
    imp(import("dictionary-nb/index.aff"), import("dictionary-nb/index.dic")),

  "nn": async () =>
    imp(import("dictionary-nn/index.aff"), import("dictionary-nn/index.dic")),

  "sv": async () =>
    imp(import("dictionary-sv/index.aff"), import("dictionary-sv/index.dic")),

  "fo": async () =>
    imp(import("dictionary-fo/index.aff"), import("dictionary-fo/index.dic")),

  "nl": async () =>
    imp(import("dictionary-nl/index.aff"), import("dictionary-nl/index.dic")),

  "hu": async () =>
    imp(import("dictionary-hu/index.aff"), import("dictionary-hu/index.dic")),

  "ro": async () =>
    imp(import("dictionary-ro/index.aff"), import("dictionary-ro/index.dic"))
}

export default DICTIONARIES

async function imp(aff: any, dic: any) {
  if (Array.isArray(dic)) {
    const dics: string[] = []
    for (const d of dic) {
      dics.push(await url(d))
    }
    return { aff: await url(aff), dic: dics }
  } else {
    return { aff: await url(aff), dic: await url(dic) }
  }
}
