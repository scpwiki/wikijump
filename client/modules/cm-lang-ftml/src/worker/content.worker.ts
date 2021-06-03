import { Prism } from "wj-prism"
import { decode, expose, ModuleProxy, transfer } from "worker-module/src/worker-lib"
import * as Spellchecker from "./content-spellcheck"

// -- MODULE

const module = {
  extract(raw: ArrayBuffer) {
    return transfer(extractContent(decode(raw)))
  },

  stats(raw: ArrayBuffer) {
    const str = decode(raw)
    const content = extractContent(str)
    const words = content.trim().split(/\s+/).length
    const bytes = raw.byteLength
    return { words, bytes }
  },

  async setSpellchecker(wasmURL: string, dictURL: string, bigramURL?: string) {
    await Spellchecker.setSpellchecker(wasmURL, dictURL, bigramURL)
  },

  async spellcheck(raw: ArrayBuffer) {
    return await Spellchecker.check(decode(raw))
  },

  async spellcheckWords(raw: ArrayBuffer) {
    return await Spellchecker.checkWords(decode(raw))
  },

  appendToDictionary(input: string | string[], frequency = 1000) {
    Spellchecker.appendToDictionary(input, frequency)
  }
}

export type ContentModuleInterface = ModuleProxy<typeof module>

expose(module)

// -- FUNCTIONS

/**
 * Extracts the actual "content" of Wikitext using the Prism grammar as a
 * parser. Replaces all other markup with spaces in order to preserve a
 * mapping between the emitted string and the original document.
 *
 * @param str - The wikitext to extract the content out of.
 */
function extractContent(str: string) {
  const tokens = Prism.tokenize(str, Prism.languages.ftml)
  let output = ""
  for (const token of tokens) {
    if (typeof token === "string") output += token
    else output += " ".repeat(token.length)
  }
  return output
}
