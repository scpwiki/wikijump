import type { TreeFragment } from "@lezer/common"
import { Range } from "./types"

/**
 * The region of a document that should be parsed, along with other
 * information such as what the edited range of the document was.
 */
export class ParseRegion {
  /** The parser should start before or at this position. */
  declare from: number

  /** The parse should stop past or at this position. */
  declare to: number

  /** The length of the region. */
  get length() {
    return this.to - this.from
  }

  declare document: {
    from: number
    to: number
    length: number
  }

  declare original: {
    from: number
    to: number
    length: number
  }

  /** The edited range of the document. */
  declare edit?: {
    /** The start of the edited range. */
    from: number
    /** The end of the edited range. */
    to: number
    /** The number of characters added in the change. */
    offset: number
  }

  constructor(rangeDocument: Range, rangeParse: Range, fragments?: TreeFragment[]) {
    this.from = Math.max(rangeDocument.from, rangeParse.from)
    this.to = Math.min(rangeDocument.to, rangeParse.to)

    this.original = { from: this.from, to: this.to, length: this.length }

    this.document = {
      from: rangeDocument.from,
      to: rangeDocument.to,
      length: rangeDocument.to - rangeDocument.from
    }

    // get the edited range of the document,
    // spanning from the start of the first edit to the end of the last edit
    if (fragments?.length) {
      let from: number, to: number, offset: number

      if (fragments.length === 1) {
        const fragment = fragments[0]
        from = fragment.openStart ? this.from : fragment.to
        to = fragment.openStart ? fragment.from : this.to
        offset = -fragment.offset
      } else {
        const reversed = [...fragments].reverse()
        const first = reversed.find(f => !f.openStart && f.openEnd) || fragments[0]
        const last = reversed.find(f => f.openStart && !f.openEnd) || reversed[0]

        from = first.openStart && first.openEnd ? first.from : first.to
        to = last.openStart && last.openEnd ? last.to : last.from

        // not sure why this is needed, something I don't understand about fragments
        // usually if this is the case the parse was interrupted, and is being continued
        if (from > to) to = this.to

        offset = -last.offset
      }

      this.edit = { from, to, offset }
    }
  }
}
