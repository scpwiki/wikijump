import * as FTML from "../vendor/ftml"
import { freeTracked, ready, trk } from "./base"
import { parse } from "./interface"

/** Converts a string of wikitext into a pretty-printed list of tokens. */
export function inspectTokens(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")
  try {
    str = preprocess ? FTML.preprocess(str) : str

    const tokenized = trk(FTML.tokenize(str))
    const tokens = tokenized.tokens()

    freeTracked()

    let out = ""
    for (const {
      slice,
      span: { start, end },
      token
    } of tokens) {
      const tokenStr = String(token.padEnd(16))
      const startStr = String(start).padStart(4, "0")
      const endStr = String(end).padStart(4, "0")
      const sliceStr = slice.slice(0, 40).replaceAll("\n", "\\n")
      out += `[${startStr} <-> ${endStr}]: ${tokenStr} => '${sliceStr}'\n`
    }

    return out
  } catch (err) {
    freeTracked()
    throw err
  }
}

/**
 * Gets a word count from a string of wikitext.
 *
 * @param str - Wikitext to count words in.
 * @param preprocess - Whether to preprocess the wikitext before counting.
 */
export function wordCount(str: string, preprocess = true) {
  if (!ready) throw new Error("FTML wasn't ready yet!")

  str = preprocess ? FTML.preprocess(str) : str
  const tree = parse(str, undefined, "draft").ast
  let count = 0

  const addWords = (str: string) => {
    str = str.trim()
    if (str.length && /\w/.test(str)) {
      count += str.split(/\s+/).length
    }
  }

  const nest = (node: (FTML.IElement | FTML.ISyntaxTree)[]) => {
    for (const child of node) {
      traverse(child)
    }
  }

  const traverse = (node: any) => {
    if ("element" in node && node.element === "text") addWords(node.data)

    if ("data" in node) {
      if (node.data?.elements) nest(node.data.elements)
      if (node.data?.items) nest(node.data.items)
      if (node.data?.rows) nest(node.data.rows)
      if (node.data?.cells) nest(node.data.cells)

      if (node.data?.label) {
        if (typeof node.data?.label === "string") addWords(node.data.label)
        else if (node.data?.label?.text) addWords(node.data.label.text)
      }
    }

    if ("elements" in node) nest(node.elements)
    if ("footnotes" in node) nest(node.footnotes)
    if ("table-of-contents" in node) nest(node["table-of-contents"])
  }

  traverse(tree)

  return count
}
