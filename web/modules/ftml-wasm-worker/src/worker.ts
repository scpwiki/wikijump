import { Comlink } from "@wikijump/comlink"
import type * as FTML from "@wikijump/ftml-wasm"
import {
  detailRenderHTML,
  detailRenderText,
  getUTF16IndexMap,
  init,
  inspectTokens,
  loading,
  makeInfo,
  Page,
  parse,
  preprocess,
  renderHTML,
  renderText,
  tokenize,
  version,
  warnings,
  wordCount
} from "@wikijump/ftml-wasm"
import * as indent from "../vendor/indent"

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

export interface FTMLModule extends Methods<typeof FTML> {
  formatHTML: typeof formatHTML
  waitUntilReady: () => void
}

const module: FTMLModule = {
  init,
  makeInfo,
  version,
  preprocess,
  tokenize,
  parse,
  renderHTML,
  detailRenderHTML,
  renderText,
  detailRenderText,
  warnings,
  getUTF16IndexMap,
  inspectTokens,
  formatHTML,
  wordCount,
  Page,
  async waitUntilReady() {
    await loading
  }
}

Comlink.expose(module)
