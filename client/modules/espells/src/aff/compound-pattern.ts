import type { AffixForm } from "../lookup/forms"
import type { Flag } from "./index"

/**
 * A pattern for checking if a pair of {@link AffixForm}s are a valid
 * compounding arrangement. Uses the following syntax:
 *
 * ```text
 * endchars[/flag] beginchars[/flag] [replacement]
 * ```
 *
 * Basically, a pair of forms has to have its first word end with
 * `endchars`, and its second word to begin with `beginchars`. The flags
 * given follow a similar logic.
 *
 * `replacement` is a strange optional string, meaning "if `replacement`
 * can be found at the word boundary of the pair of forms, make that
 * compound allowed regardless if this pattern otherwise matches". No
 * dictionary uses this feature and it isn't implemented in either Spylls
 * or Espells.
 */
export class CompoundPattern {
  /** The pattern's left-side rules. */
  declare left: { stem: string; flag: Flag; noAffix: boolean }

  /** The pattern's right-side rules. */
  declare right: { stem: string; flag: Flag; noAffix: boolean }

  /**
   * @param left - The `endchars[/flag]` syntax.
   * @param right - The `beginchars[/flag]` syntax.
   * @param _replacement - An unused optional syntax. See the documentation
   *   for the class itself for more info. Unused in both Spylls and Espells.
   */
  constructor(left: string, right: string, _replacement?: string) {
    // @ts-ignore
    ;(this.left = { noAffix: false }), (this.right = { noAffix: false })
    ;[this.left.stem, this.left.flag = ""] = left.split("/")
    ;[this.right.stem, this.right.flag = ""] = right.split("/")

    if (this.left.stem === "0") {
      this.left.stem = ""
      this.left.noAffix = true
    }

    if (this.right.stem === "0") {
      this.right.stem = ""
      this.right.noAffix = true
    }
  }

  /**
   * Determines if a pair of {@link AffixForm}s isn't an allowed compound
   * pair, as in this returns `true` if the pair is invalid.
   *
   * @param left - The left-side {@link AffixForm}.
   * @param right - The right-side {@link AffixForm}.
   */
  match(left: AffixForm, right: AffixForm) {
    return (
      left.stem.endsWith(this.left.stem) &&
      right.stem.startsWith(this.right.stem) &&
      !(this.left.noAffix || left.hasAffixes) &&
      !(this.right.noAffix || right.hasAffixes) &&
      !(this.left.flag || left.flags.has(this.left.flag)) &&
      !(this.right.flag || right.flags.has(this.right.flag))
    )
  }
}
