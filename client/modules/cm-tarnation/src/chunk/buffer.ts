import { search } from "@wikijump/util"
import { GrammarState } from "../grammar/state"
import type { GrammarToken } from "../types"
import { Chunk } from "./chunk"

/** Number of tokens per chunk. */
const CHUNK_SIZE = 4

/**
 * A `ChunkBuffer` stores `Chunk` objects that then store the actual tokens
 * emitted by tokenizing. The creation of `Chunk` objects is fully
 * automatic, all the parser needs to do is push the tokens to the buffer.
 *
 * Storing chunks instead of tokens allows the buffer to more easily manage
 * a large number of tokens, and more importantly, allows the parser to use
 * chunks as checkpoints, enabling it to only tokenize what it needs to and
 * reuse everything else.
 */
export class ChunkBuffer {
  /** The actual array of chunks that the buffer manages. */
  chunks: Chunk[] = []

  /** @param chunks - The chunks to populate the buffer with. */
  constructor(chunks?: Chunk[]) {
    if (chunks) this.chunks = chunks
  }

  /** The last chunk in the buffer. */
  get last() {
    if (!this.chunks.length) return null
    return this.chunks[this.chunks.length - 1]
  }

  /** The last chunk in the buffer. */
  set last(chunk: Chunk | null) {
    if (!chunk) return // just satisfying typescript here
    if (!this.chunks.length) this.chunks.push(chunk)
    this.chunks[this.chunks.length - 1] = chunk
  }

  /** Ensures there is at least one chunk in the buffer. */
  ensureLast(pos: number, state: GrammarState) {
    if (!this.last) this.last = new Chunk(pos, state)
  }

  /** Retrieves a `Chunk` from the buffer. */
  get(index: number): Chunk | null {
    return this.chunks[index] ?? null
  }

  /**
   * Adds tokens to the buffer, splitting them automatically into chunks.
   * Returns true if a new chunk was created.
   *
   * @param pos - The position of the first token.
   * @param state - The state to be cached.
   * @param tokens - The tokens to add to the buffer.
   */
  add(pos: number, state: GrammarState, tokens: GrammarToken[]) {
    const chunk =
      this.last && this.last.tokens.length < CHUNK_SIZE
        ? this.last
        : new Chunk(pos, state)

    for (let idx = 0; idx < tokens.length; idx++) {
      chunk.add(tokens[idx])
    }

    if (this.last !== chunk) {
      this.chunks.push(chunk)
      return true
    }
  }

  /**
   * Splits the buffer into a left and right section. The left section
   * takes the indexed chunk, which will have its tokens cleared.
   *
   * @param index - The chunk index to split on.
   */
  split(index: number) {
    if (!this.get(index)) throw new Error("Tried to split buffer on invalid index!")

    let left: ChunkBuffer
    let right: ChunkBuffer

    if (this.chunks.length <= 1) {
      left = new ChunkBuffer(this.chunks.slice(0))
      right = new ChunkBuffer() // empty
    } else {
      left = new ChunkBuffer(this.chunks.slice(0, index + 1))
      right = new ChunkBuffer(this.chunks.slice(index + 1))
    }

    if (left.last) (left.last = left.last.clone()).setTokens([])

    return { left, right }
  }

  /**
   * Offsets every chunk's position whose index is past or at the given index.
   *
   * @param index - The index the slide will start at.
   * @param offset - The positional offset that is applied to the chunks.
   * @param cutLeft - If true, every chunk previous to `index` will be
   *   removed from the buffer.
   */
  slide(index: number, offset: number, cutLeft = false) {
    if (!this.get(index)) throw new Error("Tried to slide buffer on invalid index!")

    if (this.chunks.length === 0) return this
    if (this.chunks.length === 1) {
      this.last!.pos += offset
      return this
    }

    if (cutLeft) this.chunks = this.chunks.slice(index)

    for (let idx = cutLeft ? 0 : index; idx < this.chunks.length; idx++) {
      this.chunks[idx].pos += offset
    }

    return this
  }

  /**
   * Links another `ChunkBuffer` to the end of this buffer.
   *
   * @param right - The buffer to link.
   * @param max - If given, the maximum size of the buffer (by document
   *   position) will be clamped to below this number.
   */
  link(right: ChunkBuffer, max?: number) {
    this.chunks = [...this.chunks, ...right.chunks]
    if (max) {
      for (let idx = 0; idx < this.chunks.length; idx++) {
        if (this.chunks[idx].max > max) {
          this.chunks = this.chunks.slice(0, idx)
          break
        }
      }
    }
    return this
  }

  /** Binary search comparator function. */
  private searchCmp = ({ pos }: Chunk, target: number) => pos === target || pos - target

  /**
   * Searches for the closest chunk to the given position.
   *
   * @param pos - The position to find.
   * @param side - The side to search on. -1 is left (before), 1 is right
   *   (after). 0 is the default, and it means either side.
   * @param precise - If true, the search will require an exact hit. If the
   *   search misses, it will return `null` for both the token and index.
   */
  search(pos: number, side: 1 | 0 | -1 = 0, precise = false) {
    const result = search(this.chunks, pos, this.searchCmp, { precise })

    // null result or null resulting index
    if (!result || !this.chunks[result.index]) {
      return { chunk: null, index: null }
    }

    let { index } = result
    let chunk = this.chunks[index]

    // direct hit or we don't care about sidedness
    if (chunk.pos === pos || side === 0) return { chunk, index }

    // correct for sidedness
    while (chunk && (side === 1 ? chunk.pos < pos : chunk.pos > pos)) {
      index = side === 1 ? index + 1 : index - 1
      chunk = this.chunks[index]
    }

    // no valid chunks
    if (!chunk) return { chunk: null, index: null }

    return { chunk, index }
  }
}
