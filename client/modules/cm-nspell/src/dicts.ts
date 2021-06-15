import { url } from "wj-util"
import type { DictionaryImporter } from "./types"

/** Table of dictionary import functions, using locale language codes as keys. */
export const DICTIONARIES: Record<string, DictionaryImporter> = {
  "en": async () =>
    imp(import("dictionary-en/index.aff?url"), import("dictionary-en/index.dic?url")),

  "de": async () =>
    imp(import("../vendor/de.aff?url"), [
      import("../vendor/de-transam.dic?url"),
      import("../vendor/de-bjoern.dic?url"),
      import("../vendor/de-chrome.dic?url")
    ]),

  "es": async () =>
    imp(import("dictionary-es/index.aff?url"), import("dictionary-es/index.dic?url")),

  "fr": async () =>
    imp(import("dictionary-fr/index.aff?url"), import("dictionary-fr/index.dic?url")),

  "it": async () =>
    imp(import("dictionary-it/index.aff?url"), import("dictionary-it/index.dic?url")),

  "ru": async () =>
    imp(import("dictionary-ru/index.aff?url"), import("dictionary-ru/index.dic?url")),

  "ko": async () =>
    imp(import("dictionary-ko/index.aff?url"), import("dictionary-ko/index.dic?url")),

  "pl": async () =>
    imp(import("dictionary-pl/index.aff?url"), import("dictionary-pl/index.dic?url")),

  "uk": async () =>
    imp(import("dictionary-uk/index.aff?url"), import("dictionary-uk/index.dic?url")),

  "pt": async () =>
    imp(import("dictionary-pt/index.aff?url"), import("dictionary-pt/index.dic?url")),

  "cs": async () =>
    imp(import("dictionary-cs/index.aff?url"), import("dictionary-cs/index.dic?url")),

  "vi": async () =>
    imp(import("dictionary-vi/index.aff?url"), import("dictionary-vi/index.dic?url")),

  "el": async () =>
    imp(import("dictionary-el/index.aff?url"), import("dictionary-el/index.dic?url")),

  "tr": async () =>
    imp(import("dictionary-tr/index.aff?url"), import("dictionary-tr/index.dic?url")),

  "da": async () =>
    imp(import("dictionary-da/index.aff?url"), import("dictionary-da/index.dic?url")),

  "nb": async () =>
    imp(import("dictionary-nb/index.aff?url"), import("dictionary-nb/index.dic?url")),

  "nn": async () =>
    imp(import("dictionary-nn/index.aff?url"), import("dictionary-nn/index.dic?url")),

  "sv": async () =>
    imp(import("dictionary-sv/index.aff?url"), import("dictionary-sv/index.dic?url")),

  "fo": async () =>
    imp(import("dictionary-fo/index.aff?url"), import("dictionary-fo/index.dic?url")),

  "nl": async () =>
    imp(import("dictionary-nl/index.aff?url"), import("dictionary-nl/index.dic?url")),

  "hu": async () =>
    imp(import("dictionary-hu/index.aff?url"), import("dictionary-hu/index.dic?url")),

  "ro": async () =>
    imp(import("dictionary-ro/index.aff?url"), import("dictionary-ro/index.dic?url"))
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
