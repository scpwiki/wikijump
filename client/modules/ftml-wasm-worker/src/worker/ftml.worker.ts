/* Web-workerized version of "ftml-wasm". */

import * as FTML from "ftml-wasm"
import { expose, Transfer, encode, decode } from "./lib"

const ready = FTML.loading

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

  async render(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    const { html, style } = FTML.render(str)
    const htmlBuffer = encode(html)
    const styleBuffer = encode(style)
    return Transfer([htmlBuffer, styleBuffer], [htmlBuffer, styleBuffer])
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
