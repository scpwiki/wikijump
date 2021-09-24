import { defineElement, hover } from "../../util"

export class FootnoteMarker extends HTMLButtonElement {
  static tag = "wj-footnote-ref-marker"

  constructor() {
    super()

    hover(this, {
      alsoOnFocus: true,
      on: () => {
        this.contentsElement.classList.add("is-hovered")
        // TODO: popper
      },
      off: () => {
        this.contentsElement.classList.remove("is-hovered")
        // TODO: popper
      }
    })
  }

  get refID() {
    return parseInt(this.dataset.refID ?? "0", 10)
  }

  get contentID() {
    return parseInt(this.dataset.contentID ?? "0", 10)
  }

  get contentsElement() {
    if (!this.parentElement) throw new Error("No parent element")
    const element = this.parentElement.querySelector(".wj-footnote-ref-contents")
    if (!element) throw new Error("No contents element")
    return element
  }
}

defineElement(FootnoteMarker.tag, FootnoteMarker, { extends: "button" })
