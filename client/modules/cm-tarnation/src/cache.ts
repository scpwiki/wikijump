import { Tree } from "@lezer/common"
import type { Chunk, TokenizerBuffer } from "./tokenizer"
import type { CacheMap, SerializedParserContext } from "./types"

/**
 * Cache object for a Tarnation language. Effectively is a wrapper around a
 * `WeakMap`. This cache has two mappings:
 *
 * 1. `Tree` maps to a `TokenizerBuffer`
 * 2. `Chunk` maps to a `SerializedParserContext`
 *
 * Recovering a `TokenizerBuffer` from a `Tree` allows the tokenizer to
 * reuse its previous tokenization of the tree's respective document.
 *
 * Recovering a `SerializedParserContext` allows the parser to reuse its
 * previous parse of the tokenizer's output, which is emitted into `Chunk` objects.
 */
export class Cache {
  private map = new WeakMap() as CacheMap

  /** Associates a parser context with a chunk. */
  attach(context: SerializedParserContext, chunk: Chunk): void
  /** Associates a tokenizer buffer with a tree. */
  attach(buffer: TokenizerBuffer, tree: Tree): void
  attach(value: TokenizerBuffer | SerializedParserContext, key: Tree | Chunk): void {
    this.map.set(key, value)
  }

  /** Checks if a parser context is associated with the given chunk. */
  has(chunk: Chunk): boolean
  /** Checks if a tokenizer buffer is associated with the given tree. */
  has(tree: Tree): boolean
  has(node: Tree | Chunk): boolean {
    return this.map.has(node)
  }

  /** Gets the parser context associated with the chunk, if it exists. */
  get(chunk: Chunk): SerializedParserContext | undefined
  /** Gets the tokenizer buffer associated with the tree, if it exists. */
  get(tree: Tree): TokenizerBuffer | undefined
  get(node: Tree | Chunk): TokenizerBuffer | SerializedParserContext | undefined {
    return this.map.get(node)
  }

  /**
   * Returns the first tokenizer buffer found within a tree, if any.
   *
   * @param tree - The tree to search through, recursively.
   * @param from - The start of the search area.
   * @param to - The end of the search area.
   * @param offset - An offset added to the tree's positions, so that they
   *   may match some other source's positions.
   */
  find(tree: Tree, from: number, to: number, offset = 0): TokenizerBuffer | null {
    const bundle =
      offset >= from && offset + tree.length >= to ? this.get(tree) : undefined

    if (bundle) return bundle

    // recursively check children
    for (let i = tree.children.length - 1; i >= 0; i--) {
      const child = tree.children[i]
      const pos = offset + tree.positions[i]
      if (!(child instanceof Tree && pos < to)) continue
      const found = this.find(child, from, to, pos)
      if (found) return found
    }

    return null
  }
}
