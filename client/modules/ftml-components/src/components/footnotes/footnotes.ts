import * as Popper from "@popperjs/core"
import { defineElement, hover } from "../../util"

export class FootnoteMarker extends HTMLButtonElement {
  static tag = "wj-footnote-ref-marker"

  declare onTimer?: number
  declare offTimer?: number
  declare popperInstance?: Popper.Instance

  constructor() {
    super()

    const parent = this.parentElement
    if (!parent) throw new Error("No parent element")

    hover(parent, {
      alsoOnFocus: true,
      on: () => {
        clearTimeout(this.offTimer)
        this.onTimer = setTimeout(() => this.whenHovered(), 50)
      },
      off: () => {
        clearTimeout(this.onTimer)
        this.offTimer = setTimeout(() => this.whenUnhovered(), 50)
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

  private whenHovered() {
    this.contentsElement.classList.add("is-hovered")
    if (!this.popperInstance) {
      // @ts-ignore Popper has some bad typings (Element !== HTMLElement)
      this.popperInstance = Popper.createPopper(this, this.contentsElement, {
        placement: "bottom"
      })
    }
  }

  private whenUnhovered() {
    this.contentsElement.classList.remove("is-hovered")
    if (this.popperInstance) {
      this.popperInstance.destroy()
      this.popperInstance = undefined
    }
  }
}

defineElement(FootnoteMarker.tag, FootnoteMarker, { extends: "button" })
