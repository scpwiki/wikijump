import type { GrammarState } from "../grammar/state"
import type { EmbedToken, SerializedTokenizerContext } from "../types"

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

  clone() {
    return new TokenizerContext(this.pos, this.state.clone(), this.embedded)
  }

  /** Serializes the context. */
  serialize(): SerializedTokenizerContext {
    return { pos: this.pos, state: this.state.clone(), embedded: this.embedded }
  }

  /** Deserializes a serialized context and returns a new `TokenizerContext`. */
  static deserialize(serialized: SerializedTokenizerContext) {
    return new TokenizerContext(
      serialized.pos,
      serialized.state.clone(),
      serialized.embedded
    )
  }
}
