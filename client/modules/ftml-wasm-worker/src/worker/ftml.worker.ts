import * as FTML from "ftml-wasm"
import type * as Binding from "ftml-wasm/vendor/ftml"
import type { TransferDescriptor } from "threads"
import {
  decode,
  encode,
  expose,
  transfer,
  transferMultiple
} from "threads-worker-module/src/worker-lib"
import type { ModuleProxy } from "threads/dist/types/master"
import * as indent from "../../vendor/indent"

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
  async init(wasmURL?: Binding.InitInput) {
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

  async parse(raw: ArrayBuffer, info?: FTML.PartialInfo) {
    await ready
    const str = decode(raw)
    return FTML.parse(str, info)
  },

  async renderHTML(raw: ArrayBuffer, info?: FTML.PartialInfo, format = false) {
    await ready
    const str = decode(raw)
    let { html, styles } = FTML.renderHTML(str, info)
    if (format) html = formatHTML(html)
    const htmlBuffer = encode(html)
    const styleBuffer = styles.map(style => encode(style))
    return transferMultiple(
      [htmlBuffer, styleBuffer],
      [htmlBuffer, ...styleBuffer]
    ) as TransferDescriptor<[html: ArrayBuffer, styles: ArrayBuffer[]]>
  },

  async detailRenderHTML(raw: ArrayBuffer, info?: FTML.PartialInfo) {
    await ready
    const str = decode(raw)
    return FTML.detailRenderHTML(str, info)
  },

  async renderText(raw: ArrayBuffer, info?: FTML.PartialInfo) {
    await ready
    const str = decode(raw)
    const text = FTML.renderText(str, info)
    return transfer(text)
  },

  async detailRenderText(raw: ArrayBuffer, info?: FTML.PartialInfo) {
    await ready
    const str = decode(raw)
    return FTML.detailRenderText(str, info)
  },

  async warnings(raw: ArrayBuffer, info?: FTML.PartialInfo) {
    await ready
    const str = decode(raw)
    return FTML.warnings(str, info)
  },

  async inspectTokens(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    return transfer(FTML.inspectTokens(str))
  }
}

export type FTMLWorkerInterface = ModuleProxy<typeof module>

expose(module)
