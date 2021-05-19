/* Web-workerized version of "ftml-wasm". */

import * as FTML from "ftml-wasm"
// untyped import
// @ts-ignore
import indent from "indent.js"
import { decode, encode, expose, Transfer } from "./lib"

const ready = FTML.loading

const INLINE = [
  "b",
  "del",
  "em",
  "i",
  "ins",
  "q",
  "s",
  "small",
  "strong",
  "sub",
  "sup",
  "u",
  "tt",
  "mark"
]

/**
 * Ad-hoc HTML formatting.
 *
 * Creates newlines between any non-obvious inline-formatting tags. Uses
 * `indent.js`, a tiny library, to handle indentation of the resulting HTML.
 */
function formatHTML(html: string) {
  try {
    html = html
      .replaceAll(/<\/?([^\s<>]+)([^]*?)>/g, (match, tag, extra) =>
        !INLINE.includes(tag) || extra?.length ? `\n${match}\n` : match
      )
      // Remove blank lines
      .replaceAll(/^\s*?\n/gm, "")
    return indent.html(html, { tabString: "  " })
  } catch (error) {
    return html
  }
}

expose({
  async init(wasmURL: string) {
    await FTML.init(wasmURL)
    return true
  },

  async version() {
    await ready
    return Transfer(encode(FTML.version()))
  },

  async preprocess(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return Transfer(encode(FTML.preprocess(str)))
  },

  async tokenize(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return FTML.tokenize(str)
  },

  async parse(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return FTML.parse(str)
  },

  async render(raw: ArrayBuffer, format = false) {
    await ready
    const str = decode(raw)
    let { html, styles } = FTML.render(str)
    if (format) html = formatHTML(html)
    const htmlBuffer = encode(html)
    const styleBuffer = styles.map(style => encode(style))
    return Transfer([htmlBuffer, styleBuffer], [htmlBuffer, ...styleBuffer])
  },

  async renderText(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    const text = FTML.render(str, { mode: "text" })
    return Transfer(encode(text))
  },

  async detailedRender(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return FTML.detailedRender(str)
  },

  async warnings(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return FTML.warnings(str)
  },

  async inspectTokens(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return Transfer(encode(FTML.inspectTokens(str)))
  }
})
