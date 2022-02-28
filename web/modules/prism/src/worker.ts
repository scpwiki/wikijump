import { Comlink } from "@wikijump/comlink"
import type PrismType from "prismjs"
import "../vendor/prism"
import { prismBase } from "../vendor/prism-langs"
import { prismSvelte } from "../vendor/prism-svelte"
import { prismFTML } from "./ftml"

/** Reference to the Prism syntax highlighter. */
export const Prism: typeof PrismType = globalThis.Prism

// add languages
prismBase(Prism)
prismSvelte(Prism)
prismFTML(Prism)

// set prism class prefix
// https://prismjs.com/plugins/custom-class/
Prism.plugins.customClass.prefix("wj-code-")

// yoink Prism's encode function so that we can escape strings identically
const encode: (src: string) => string = Prism.util.encode as any

const RAW_LANGS = ["raw", "text", "none", ""]

const module = {
  /** Returns the list of languages (and their aliases) supported by Prism. */
  getLanguages() {
    return Object.keys(Prism.languages).filter(
      lang => typeof Prism.languages[lang] !== "function"
    )
  },
  /**
   * Highlights a string of code and returns HTML, given a specified
   * language. If the language specified isn't known by Prism, the string
   * of code will be escaped and returned with no syntax highlighting.
   *
   * If the given language is `raw`, `text`, `none`, or an empty string,
   * the string of code will be escaped and returned as is.
   *
   * @param code - The string to be highlighted.
   * @param lang - The language to highlight the code with.
   */
  highlight(code: string, lang: string) {
    try {
      if (lang && !RAW_LANGS.includes(lang) && lang in Prism.languages) {
        const grammar = Prism.languages[lang]
        const html = Prism.highlight(code, grammar, lang)
        return html
      }
    } catch {}

    // fallback to just returning the escaped code
    return encode(code)
  },

  /** Sets or gets the status of Prism's `manual` property. */
  manual(set?: boolean) {
    if (typeof set !== "undefined") Prism.manual = set
    return Prism.manual
  },

  /** Sets or gets the status of Prism's `disableWorkerMessageHandler` property. */
  disableWorkerMessageHandler(set?: boolean) {
    if (typeof set !== "undefined") Prism.disableWorkerMessageHandler = set
    return Prism.disableWorkerMessageHandler ?? false
  }
}

export type PrismModule = typeof module

Comlink.expose(module)
