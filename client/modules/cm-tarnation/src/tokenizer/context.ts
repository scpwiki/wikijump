import type { GrammarState } from "../grammar/state"
import type { NestToken } from "../types"

/** Context/state object for the tokenizer. */
export class TokenizerContext {
  /**
   * @param pos - The document position of the tokenizer.
   * @param state - The grammar's state.
   * @param nested - Nested language, if present.
   */
  constructor(
    public pos: number,
    public state: GrammarState,
    public nested: null | [lang: string, start: number] = null
  ) {}

  /**
   * Starts the nesting of a language.
   *
   * @param lang - The language to be nested.
   * @param start - The start position of the nested language.
   */
  startNested(lang: string, start: number) {
    this.nested = [lang, start]
  }

  /**
   * Stops the nesting of the currently nested language, and returns the
   * finalized range. Returns null if no language was being nested.
   *
   * @param end - The end position of the nested language.
   */
  endNested(end: number): NestToken | null {
    if (!this.nested) return null
    const embedded = this.nested
    this.nested = null
    return [...embedded, end]
  }

  /**
   * Returns if this context is equal to another.
   *
   * @param other - The other context to compare to.
   * @param offset - The offset to apply to the other context when
   *   comparing positions.
   */
  equals(other: TokenizerContext, offset = 0) {
    if (this.pos !== other.pos + offset || !this.state.equals(other.state)) return false
    if (Boolean(this.nested) !== Boolean(other.nested)) return false
    if (this.nested) {
      if (this.nested[0] !== other.nested![0]) return false
      if (this.nested[1] !== other.nested![1]) return false
    }
    return true
  }

  /** Returns a clone of this context. */
  clone() {
    return new TokenizerContext(this.pos, this.state.clone(), this.nested)
  }
}
