import * as Popper from "@popperjs/core"
import { defineElement, hover } from "../../util"

// TODO: proper mobile support (need more infrastructure for mobile support)

/**
 * FTML `[[footnote]]` marker element. Handles placement and visibility of
 * the footnote tooltip, and clicking to scroll to the footnotes block.
 */
export class FootnoteReferenceMarker extends HTMLButtonElement {
  static tag = "wj-footnote-ref-marker"

  /** Timer to keep track of the delay for revealing the tooltip. */
  declare onTimer?: number

  /** Timer to keep track of the delay for hiding the tooltip. */
  declare offTimer?: number

  /** The Popper.js instance for handling placement of the tooltip. */
  declare popperInstance?: Popper.Instance

  constructor() {
    super()

    hover(this.parent, {
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

  /** The parent element of the marker. */
  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  /** The numeric ID of the footnote. */
  get footnoteID() {
    return parseInt(this.dataset.id ?? "0", 10)
  }

  /** Get the tooltip element for this marker. */
  get tooltip() {
    const element = this.parent.querySelector(".wj-footnote-ref-tooltip")
    if (!element) throw new Error("No contents element")
    return element
  }

  /** Finds this footnote's corresponding list-item in the first footnotes block. */
  private findFootnote() {
    const body = this.closest(".wj-body")
    if (!body) throw new Error("No parent body")
    const footnote = body.querySelector(
      `.wj-footnote-list-item[data-id="${this.footnoteID}"]`
    )
    if (!footnote) throw new Error("No footnote")
    return footnote as HTMLElement
  }

  /** Fired when the marker is hovered over. */
  private whenHovered() {
    this.tooltip.classList.add("is-hovered")
    if (!this.popperInstance) {
      // @ts-ignore Popper has some bad typings (Element !== HTMLElement)
      this.popperInstance = Popper.createPopper(this.parent, this.tooltip, {
        placement: "bottom"
      })
    }
  }

  /** Fired when the marker is no longer being hovered over. */
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

/**
 * FTML `[[footnote]]` footnotes block list-item marker. Handles scrolling
 * to the footnote reference when clicked.
 */
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

  /** The parent element of the marker. */
  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  /** The numeric ID of the footnote. */
  get footnoteID() {
    return parseInt(this.parent.dataset.id ?? "0", 10)
  }

  /** Finds this footnote's corresponding reference in the page. */
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
