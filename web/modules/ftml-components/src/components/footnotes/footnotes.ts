import { addElement, BaseButton, BaseTooltipButton } from "@wikijump/dom"

/**
 * FTML `[[footnote]]` marker element. Handles placement and visibility of
 * the footnote tooltip, and clicking to scroll to the footnotes block.
 */
export class FootnoteRefMarkerElement extends BaseTooltipButton {
  static tag = "wj-footnote-ref-marker"

  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  get tooltip() {
    const element = this.parent.querySelector(".wj-footnote-ref-tooltip")
    if (!element) throw new Error("No contents element")
    return element as HTMLElement
  }

  whenClicked() {
    const footnote = this.findFootnote()
    footnote.scrollIntoView()
    footnote.focus()
  }

  /** Finds this footnote's corresponding list-item in the first footnotes block. */
  findFootnote() {
    const body = this.closest(".wj-body")
    if (!body) throw new Error("No parent body")
    const id = parseInt(this.dataset.id ?? "0", 10)
    const footnote = body.querySelector(`.wj-footnote-list-item[data-id="${id}"]`)
    if (!footnote) throw new Error("No footnote")
    return footnote as HTMLElement
  }
}

/**
 * FTML `[[footnote]]` footnotes block list-item marker. Handles scrolling
 * to the footnote reference when clicked.
 */
export class FootnoteListMarkerElement extends BaseButton {
  static tag = "wj-footnote-list-item-marker"

  whenClicked() {
    const footnote = this.findFootnote()
    footnote.scrollIntoView()
    footnote.focus()
  }

  /** Finds this footnote's corresponding reference in the page. */
  findFootnote() {
    const body = this.closest(".wj-body")
    if (!body) throw new Error("No parent body")
    const id = parseInt(this.parentElement?.dataset.id ?? "0", 10)
    const footnote = body.querySelector(`.wj-footnote-ref-marker[data-id="${id}"]`)
    if (!footnote) throw new Error("No footnote")
    return footnote as HTMLElement
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-footnote-ref-marker": FootnoteRefMarkerElement
    "wj-footnote-list-item-marker": FootnoteListMarkerElement
  }

  interface Window {
    FootnoteRefMarkerElement: typeof FootnoteRefMarkerElement
    FootnoteListMarkerElement: typeof FootnoteListMarkerElement
  }
}

addElement(FootnoteRefMarkerElement, "FootnoteRefMarkerElement")
addElement(FootnoteListMarkerElement, "FootnoteListMarkerElement")
