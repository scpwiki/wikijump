import * as Popper from "@popperjs/core"
import { defineElement, hover } from "../../util"

export class FootnoteReferenceMarker extends HTMLButtonElement {
  static tag = "wj-footnote-ref-marker"

  declare onTimer?: number
  declare offTimer?: number
  declare popperInstance?: Popper.Instance

  constructor() {
    super()

    hover(this.parent, {
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

    this.addEventListener("click", () => {
      const footnote = this.findFootnote()
      footnote.scrollIntoView()
      footnote.focus()
    })
  }

  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  get footnoteID() {
    return parseInt(this.dataset.id ?? "0", 10)
  }

  get tooltip() {
    const element = this.parent.querySelector(".wj-footnote-ref-tooltip")
    if (!element) throw new Error("No contents element")
    return element
  }

  private findFootnote() {
    const body = this.closest(".wj-body")
    if (!body) throw new Error("No parent body")
    const footnote = body.querySelector(
      `.wj-footnote-list-item[data-id="${this.footnoteID}"]`
    )
    if (!footnote) throw new Error("No footnote")
    return footnote as HTMLElement
  }

  private whenHovered() {
    this.tooltip.classList.add("is-hovered")
    if (!this.popperInstance) {
      // @ts-ignore Popper has some bad typings (Element !== HTMLElement)
      this.popperInstance = Popper.createPopper(this.parent, this.tooltip, {
        placement: "bottom"
      })
    }
  }

  private whenUnhovered() {
    this.tooltip.classList.remove("is-hovered")
    if (this.popperInstance) {
      // we'll only destroy the instance after
      // a timeout, to give room for a fade animation
      this.offTimer = setTimeout(() => {
        this.popperInstance!.destroy()
        this.popperInstance = undefined
      }, 100)
    }
  }
}

export class FootnoteListMarker extends HTMLButtonElement {
  static tag = "wj-footnote-list-item-marker"

  constructor() {
    super()

    this.addEventListener("click", () => {
      const footnote = this.findFootnote()
      footnote.scrollIntoView()
      footnote.focus()
    })
  }

  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  get footnoteID() {
    return parseInt(this.parent.dataset.id ?? "0", 10)
  }

  findFootnote() {
    const body = this.closest(".wj-body")
    if (!body) throw new Error("No parent body")
    const footnote = body.querySelector(
      `.wj-footnote-ref-marker[data-id="${this.footnoteID}"]`
    )
    if (!footnote) throw new Error("No footnote")
    return footnote as HTMLElement
  }
}

defineElement(FootnoteReferenceMarker.tag, FootnoteReferenceMarker, { extends: "button" })
defineElement(FootnoteListMarker.tag, FootnoteListMarker, { extends: "button" })
