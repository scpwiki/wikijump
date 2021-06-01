import { klona } from "klona"
import type { EmbeddedData, SerializedParserContext } from "../types"
import { ParserBuffer } from "./buffer"
import { ParserStack } from "./stack"

/**
 * Context/state object for the parser. State is stored separately from the
 * parser, so that it may be recovered from a cache later and be reused.
 */
export class ParserContext {
  /**
   * @param start - The starting document position of the parser.
   * @param index - The index of the next token to be parsed.
   * @param buffer - The parser's token buffer.
   * @param stack - The parsers's stack.
   * @param embedded - The parser's embedded language handler state.
   */
  constructor(
    public start: number,
    public index: number,
    public buffer: ParserBuffer,
    public stack: ParserStack,
    public embedded: EmbeddedData
  ) {}

  /**
   * Serializes the context.
   *
   * @param full - If true, the `ParserBuffer` will be cloned deeply
   *   instead of being a shallow clone.
   */
  serialize(full = false): SerializedParserContext {
    return {
      pos: this.start,
      index: this.index,
      buffer: full ? this.buffer.clone(true) : this.buffer.shallow(),
      stack: this.stack.serialize(),
      embedded: klona(this.embedded)
    }
  }

  /**
   * Returns a clone of the context.
   *
   * @param full - If true, the `ParserBuffer` will be cloned deeply
   *   instead of being a shallow clone.
   */
  clone(full = false) {
    return ParserContext.deserialize(this.serialize(full))
  }

  /** Deserializes a serialized context and returns a new `ParserContext`. */
  static deserialize(serialized: SerializedParserContext) {
    return new ParserContext(
      serialized.pos,
      serialized.index,
      new ParserBuffer(serialized.buffer),
      new ParserStack(serialized.stack),
      serialized.embedded
    )
  }
}
