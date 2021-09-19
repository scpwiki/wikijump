import type { GrammarState } from "../grammar/state"
import type { EmbedToken } from "../types"

/** Context/state object for the tokenizer. */
export class TokenizerContext {
  /**
   * @param pos - The document position of the tokenizer.
   * @param state - The grammar's state.
   * @param embedded - Embedded language data, if present.
   */
  constructor(
    public pos: number,
    public state: GrammarState,
    public embedded: null | [lang: string, start: number] = null
  ) {}

  /**
   * Starts the embedding of a language.
   *
   * @param lang - The language to be embedded.
   * @param start - The start position of the embedded language.
   */
  setEmbedded(lang: string, start: number) {
    this.embedded = [lang, start]
  }

  /**
   * Stops the embedding of the currently embedded data, and returns the
   * finalized range. Returns null if no language was being embedded.
   *
   * @param end - The end position of the embedded language.
   */
  endEmbedded(end: number): EmbedToken | null {
    if (!this.embedded) return null
    const embedded = this.embedded
    this.embedded = null
    return [...embedded, end]
  }

  equals(other: TokenizerContext, offset = 0) {
    if (this.pos !== other.pos + offset || !this.state.equals(other.state)) return false
    if (Boolean(this.embedded) !== Boolean(other.embedded)) return false
    if (this.embedded) {
      if (this.embedded[0] !== other.embedded![0]) return false
      if (this.embedded[1] !== other.embedded![1]) return false
    }
    return true
  }

  clone() {
    return new TokenizerContext(this.pos, this.state.clone(), this.embedded)
  }
}
