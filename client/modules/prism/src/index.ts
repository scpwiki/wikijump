import type PrismType from "prismjs"
import "../vendor/prism"

// Re-export a reference to Prism so that there is actually a half-decent
// way of accessing it
/** Reference to the Prism syntax highlighter. */
export const Prism: typeof PrismType = globalThis.Prism

// set prism class prefix
// https://prismjs.com/plugins/custom-class/
Prism.plugins.customClass.prefix("code-")

// yoink Prism's encode function so that we can escape strings identically
const encode: (src: string) => string = Prism.util.encode as any

const RAW_LANGS = ["raw", "text", "none", ""]

// asynchronously import languages to prevent the large prism JS file
// from blocking the page from rendering
// only the base library is synchronously imported and everything else is
// asynchronously imported
// prettier-ignore
async function importLanguages() {
  ;(await import("../vendor/prism-langs")).prismBase(Prism)
  ;(await import("../vendor/prism-svelte")).prismSvelte(Prism)
  ;(await import("./ftml")).prismFTML(Prism)
}

export const languagesReady = importLanguages()

/**
 * Highlights a string of code and returns HTML, given a specified
 * language. If the language specified isn't known by Prism, the string of
 * code will be escaped and returned with no syntax highlighting.
 *
 * If the given language is `raw`, `text`, `none`, or an empty string, the
 * string of code will be escaped and returned as is.
 *
 * @param code - The string to be highlighted.
 * @param lang - The language to highlight the code with.
 */
export function highlight(code: string, lang: string) {
  try {
    if (lang && !RAW_LANGS.includes(lang) && lang in Prism.languages) {
      const grammar = Prism.languages[lang]
      const html = Prism.highlight(code, grammar, lang)
      return html
    }
  } catch {}

  // fallback to just returning the escaped code
  return encode(code)
}
