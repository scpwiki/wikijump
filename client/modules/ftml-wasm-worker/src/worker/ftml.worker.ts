import * as FTML from "ftml-wasm"
import type * as Binding from "ftml-wasm/vendor/ftml"
// untyped import
// @ts-ignore
import indent from "indent.js"
import type { TransferDescriptor } from "threads"
import type { ModuleProxy } from "threads/dist/types/master"
import { decode, encode, expose, transfer, transferMultiple } from "./lib"

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

const module = {
  async init(wasmURL: Binding.InitInput) {
    await FTML.init(wasmURL)
    return true
  },

  async version() {
    await ready
    return transfer(FTML.version())
  },

  async preprocess(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return transfer(FTML.preprocess(str))
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
    return transferMultiple(
      [htmlBuffer, styleBuffer],
      [htmlBuffer, ...styleBuffer]
    ) as TransferDescriptor<[html: ArrayBuffer, styles: ArrayBuffer[]]>
  },

  async renderText(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    const text = FTML.render(str, { mode: "text" })
    return transfer(text)
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
    return transfer(FTML.inspectTokens(str))
  }
}

export type FTMLWorkerInterface = ModuleProxy<typeof module>

expose(module)
