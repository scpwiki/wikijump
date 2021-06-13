import { url } from "wj-util"
import type { DictionaryImporter } from "./types"

/** Table of dictionary import functions, using locale language codes as keys. */
export const DICTIONARIES: Record<string, DictionaryImporter> = {
  "en": async () => ({
    aff: await url(import("dictionary-en/index.aff?url")),
    dic: await url(import("dictionary-en/index.dic?url"))
  })
}

export default DICTIONARIES
