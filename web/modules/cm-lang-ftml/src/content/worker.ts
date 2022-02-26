// organize-imports-ignore
import "@wikijump/prism/vendor/prism-min"
import { prismFTML } from "@wikijump/prism/src/ftml"
import { Comlink, encode } from "@wikijump/comlink"

// add FTML to our tiny prism instance
// @ts-ignore undeclared global
prismFTML(Prism)

// -- MODULE

const module = {
  /**
   * Extracts the actual "content" of wikitext using the Prism grammar as a
   * parser. Replaces all other markup with spaces in order to preserve a
   * mapping between the emitted string and the original document.
   *
   * @param str - The wikitext to extract the content out of.
   */
  extractContent(str: string) {
    // @ts-ignore undeclared global
    const tokens = Prism.tokenize(str, Prism.languages.ftml)
    let output = ""
    for (const token of tokens) {
      if (typeof token === "string") output += token
      else output += " ".repeat(token.length)
    }
    return output
  },

  /**
   * Gets the word count of a string of wikitext. Uses the Prism grammar as a parser.
   *
   * @param str - The wikitext to extract the content out of.
   */
  words(str: string) {
    const content = this.extractContent(str)
    const words = content.trim().split(/\s+/).length
    return words
  }
}

export type ContentModule = typeof module

Comlink.expose(module)
