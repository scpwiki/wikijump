import type { Text } from "@codemirror/state"
import * as FTML from "ftml-wasm-worker"
import { Memoize } from "typescript-memoize"
import { toFragment } from "wj-util"

export class RenderHandler {
  private declare fragmentNode?: DocumentFragment

  constructor(public doc?: Text) {}

  @Memoize()
  private get src() {
    if (!this.doc) return ""
    return this.doc.toString()
  }

  @Memoize()
  async result(format = false) {
    return await FTML.render(this.src, format)
  }

  @Memoize()
  async html(format = false) {
    const { html } = await this.result(format)
    return html
  }

  @Memoize()
  async style() {
    const { style } = await this.result()
    return style
  }

  @Memoize()
  async text() {
    const text = await FTML.renderText(this.src)
    return text
  }

  @Memoize()
  async parse() {
    return await FTML.parse(this.src)
  }

  @Memoize()
  async ast() {
    const { ast } = await this.parse()
    return ast
  }

  @Memoize()
  async warnings() {
    const { warnings } = await this.parse()
    return warnings
  }

  async fragment() {
    if (!this.fragmentNode) {
      this.fragmentNode = toFragment(await this.html())
    }
    return this.fragmentNode.cloneNode(true)
  }

  @Memoize()
  async stringifiedAST() {
    const ast = await this.ast()
    return JSON.stringify(ast, undefined, 2)
  }
}
