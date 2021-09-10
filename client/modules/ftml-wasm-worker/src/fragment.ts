import type { PartialInfo } from "ftml-wasm"
import { toFragment } from "wj-util"
import FTML from "./ftml"

export class FTMLFragment {
  private declare styles: string[]
  private declare fragment: DocumentFragment
  private declare html: string
  private declare src: string
  private declare info?: PartialInfo

  ready = false

  constructor(src: string, info?: PartialInfo) {
    this.src = src
    this.info = info
  }

  async render() {
    if (!this.ready) {
      const { html, styles } = await FTML.renderHTML(this.src, this.info)
      const fragment = toFragment(html)
      this.fragment = fragment
      this.html = html
      this.styles = styles
      this.ready = true
    }
    return this.unwrap()!
  }

  unwrap() {
    if (this.ready) {
      return {
        fragment: this.fragment.cloneNode(true),
        styles: this.styles,
        html: this.html
      }
    }
  }
}
