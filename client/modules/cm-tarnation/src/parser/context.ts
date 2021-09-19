import type { SerializedParserContext } from "../types"
import { ParserBuffer } from "./buffer"
import { ParserStack } from "./stack"

/**
 * Context/state object for the parser. State is stored separately from the
 * parser, so that it may be recovered from a cache later and be reused.
 */
export class ParserContext {
  /**
   * @param index - The index of the next token to be parsed.
   * @param buffer - The parser's token buffer.
   * @param stack - The parsers's stack.
   */
  constructor(
    public index: number = 0,
    public buffer: ParserBuffer = new ParserBuffer(),
    public stack: ParserStack = new ParserStack([])
  ) {}

  /** Serializes the context. */
  serialize(upto?: number): SerializedParserContext {
    return {
      index: this.index,
      buffer: this.buffer.shallow(upto),
      stack: this.stack.serialize()
    }
  }

  /** Returns a clone of the context. */
  clone() {
    return ParserContext.deserialize(this.serialize())
  }

  /** Deserializes a serialized context and returns a new `ParserContext`. */
  static deserialize(serialized: SerializedParserContext) {
    return new ParserContext(
      serialized.index,
      new ParserBuffer(serialized.buffer),
      new ParserStack(serialized.stack)
    )
  }
}
