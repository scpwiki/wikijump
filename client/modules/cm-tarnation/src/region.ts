import type { Input, TreeFragment } from "@lezer/common"

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

  /**
   * The range describing the original parse range when this region was
   * constructed. This is so that the `from` and `to` properties can be
   * modified while retaining originally designated parse range.
   */
  declare original: {
    /** The start of the original range. */
    from: number
    /** The end of the original range. */
    to: number
    /** The length of the original range. */
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

  /** The ranges to be parsed. */
  declare ranges: { from: number; to: number }[]

  /**
   * @param input - The input to get the parse region for.
   * @param ranges - The ranges of the document that should be parsed.
   * @param fragments - Fragments that are used to compute the edited range.
   */
  constructor(
    input: Input,
    ranges: { from: number; to: number }[],
    fragments?: TreeFragment[]
  ) {
    this.from = ranges[0].from
    this.to = Math.min(input.length, ranges[ranges.length - 1].to)
    this.original = { from: this.from, to: this.to, length: this.length }
    this.ranges = ranges

    // get the edited range of the document,
    // spanning from the start of the first edit to the end of the last edit
    if (fragments?.length) {
      let from: number, to: number, offset: number

      if (fragments.length === 1) {
        const fragment = fragments[0]
        // special case that seems to happen when scrolling,
        // the fragment is the entire parsed range
        if (fragment.offset === 0 && !fragment.openStart && fragment.openEnd) {
          from = input.length
          to = input.length
          offset = 0
        } else {
          from = fragment.openStart ? this.from : fragment.to
          to = fragment.openStart ? fragment.from : this.to
          offset = -fragment.offset
        }
      } else {
        const reversed = [...fragments].reverse()
        const first = reversed.find(f => !f.openStart && f.openEnd) || fragments[0]
        const last = fragments.find(f => f.openStart && !f.openEnd) || reversed[0]

        from = first.openStart && first.openEnd ? first.from : first.to
        to = last.openStart && last.openEnd ? last.to : last.from
        offset = -last.offset

        // not sure why this is needed, something I don't understand about fragments
        // usually if this is the case the parse was interrupted, and is being continued
        if (from > to) {
          to = from
          offset = 0
        }
      }

      this.edit = { from, to, offset }
    }
  }

  /** True if we don't need to care about range handling. */
  get contiguous() {
    return this.ranges.length === 1
  }

  /**
   * Compensates for an adjustment to a position. That is, given the range
   * `pos` is inside of, what position should adding `addition` return?
   * This is for skipping past the gaps inbetween ranges.
   *
   * @param pos - The position to start from.
   * @param addition - The amount to add to `pos`.
   */
  compensate(pos: number, addition: number): number {
    const desired = pos + addition
    if (this.ranges.length === 1) return desired

    const range = this.posRange(pos)
    if (!range) return desired

    // forwards compensation
    if (desired > range.to) {
      const next = this.posRange(pos, 1)
      if (!next) return desired
      const nextDesired = next.from + (desired - range.to) - 1
      // recursively compensate, if needed
      if (nextDesired > next.to) {
        return this.compensate(next.to, nextDesired - next.to)
      }
      return nextDesired
    }
    // backwards compensation
    else if (desired < range.from) {
      const prev = this.posRange(pos, -1)
      if (!prev) return desired
      const prevDesired = prev.to + (desired - range.from) + 1
      // recursively compensate, if needed
      if (prevDesired < prev.from) {
        return this.compensate(prev.from, prevDesired - prev.from)
      }
      return prevDesired
    }

    // no compensation needed
    return desired
  }

  /**
   * Clamps a `to` value for a range to the end of the parse range that
   * `from` is inside of.
   *
   * @param from - The `from` position, for which the `to` position will be
   *   clamped relative to.
   * @param to - The `to` position, which will be clamped to the end of the
   *   range that `from` is inside of.
   */
  clamp(from: number, to: number) {
    const range = this.posRange(from)
    if (!range) return to
    return range.to
  }

  /**
   * Gets what range the given position is inside of. Returns `null` if the
   * position can't be found inside of any range.
   *
   * @param pos - The position to get the range for.
   * @param side - The side of the range to get. -1 returns the range
   *   previous, 1 returns the range after. Defaults to 0.
   */
  posRange(pos: number, side: -1 | 0 | 1 = 0) {
    if (this.ranges.length === 1) return this.ranges[0]
    for (let i = 0; i < this.ranges.length; i++) {
      const range = this.ranges[i]
      if (pos >= range.from && pos <= range.to) {
        let final: { from: number; to: number }
        // prettier-ignore
        switch (side) {
          case -1: final = this.ranges[i - 1]; break
          case  0: final = range; break
          case  1: final = this.ranges[i + 1]; break
        }
        return final ?? null
      }
    }
    return null
  }
}
