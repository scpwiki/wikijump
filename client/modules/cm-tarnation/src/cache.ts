import { Tree } from "lezer-tree"
import { ParserContext } from "./parser"
import type { TokenizerBuffer } from "./tokenizer"
import type { SerializedParserContext } from "./types"

export class CacheBundle {
  private declare _tokenizerBuffer: TokenizerBuffer
  private declare _parserContext: SerializedParserContext

  constructor(tokenizerBuffer: TokenizerBuffer, parserContext: ParserContext) {
    this._tokenizerBuffer = tokenizerBuffer
    this._parserContext = parserContext.serialize()
  }

  get tokenizerBuffer() {
    return this._tokenizerBuffer
  }

  get parserContext() {
    return ParserContext.deserialize(this._parserContext)
  }
}

export class Cache {
  private map = new WeakMap<Tree, CacheBundle>()

  /** Creates a bundle and associates it with the given tree. */
  attach(tokenizerBuffer: TokenizerBuffer, parserContext: ParserContext, tree: Tree) {
    const bundle = new CacheBundle(tokenizerBuffer, parserContext)
    this.map.set(tree, bundle)
  }

  /** Checks if a bundle is associated with the given tree. */
  has(tree: Tree) {
    return this.map.has(tree)
  }

  /** Gets the bundle associated with the tree, if it exists. */
  get(tree: Tree) {
    return this.map.get(tree)
  }

  /**
   * Returns the first bundle found within a tree, if any.
   *
   * @param tree - The tree to search through, recursively.
   * @param from - The start of the search area.
   * @param to - The end of the search area.
   * @param offset - An offset added to the tree's positions, so that they
   *   may match some other source's positions.
   */
  find(tree: Tree, from: number, to: number, offset = 0): CacheBundle | null {
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
