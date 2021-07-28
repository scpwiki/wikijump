import type { SerializedTokenizerContext } from "../types"
import { TokenizerStack } from "./stack"

/** Context/state object for the tokenizer. */
export class TokenizerContext {
  /**
   * @param pos - The document position of the tokenizer.
   * @param stack - The tokenizer's stack.
   */
  constructor(public pos: number, public stack: TokenizerStack) {}

  /** Serializes the context. */
  serialize(): SerializedTokenizerContext {
    return { pos: this.pos, stack: this.stack.serialize() }
  }

  /** Deserializes a serialized context and returns a new `TokenizerContext`. */
  static deserialize(serialized: SerializedTokenizerContext) {
    return new TokenizerContext(serialized.pos, new TokenizerStack(serialized.stack))
  }
}
