/**
 * @file Web-workerized version of "ftml-wasm".
 */

import * as FTML from "ftml-wasm"
import { expose, transfer, decode } from "./lib"

// TODO: figure out if there is a decent way of avoiding bundling here
import module from "ftml-wasm/vendor/ftml_bg.wasm"
FTML.init(module as any)

const ready = FTML.loading

expose({
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

  async renderHTML(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    const { html } = FTML.render(str)
    return transfer(html)
  },

  async renderStyle(raw: ArrayBuffer) {
    await ready
    const str = decode(raw)
    const { style } = FTML.render(str)
    return transfer(style)
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
})
