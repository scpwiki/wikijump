import { toFragment } from "../../wj-util/dist"
import * as FTML from "./index"

export class FTMLFragment {
  private declare style: string
  private declare fragment: DocumentFragment
  private declare src: string

  ready = false

  constructor(src: string) {
    this.src = src
  }

  async render() {
    if (this.ready) return this.unwrap()!
    const { html, style } = await FTML.render(this.src)
    const fragment = toFragment(html)
    this.fragment = fragment
    this.style = style
    return { fragment, style }
  }

  unwrap() {
    if (this.ready) {
      return { fragment: this.fragment.cloneNode(), style: this.style }
    }
  }
}
