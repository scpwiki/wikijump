const END_WORD_SYMBOL = Symbol("$")
type TrieMap<T> = Map<string | symbol, TrieMap<T> | T>

/** Classic trie data structure. */
export class Trie<T> {
  /** Entrypoint for the trie. */
  private head: TrieMap<T> = new Map()

  /**
   * Adds a word to the trie, which will point to a value.
   *
   * @param word - The word to add.
   * @param addValue - A callback function for how to add the word to the
   *   trie. It is provided the current value of the word added, if any.
   *   This allows for merging overlapping values.
   */
  add(word: string, addValue?: (cur?: T) => T) {
    const current = this.traverse(word, true)
    current.set(
      END_WORD_SYMBOL,
      !addValue ? (word as any) : addValue(current.get(END_WORD_SYMBOL) as T)
    )
  }

  /** Determines if a word is in the trie. */
  has(word: string) {
    const ret = this.traverse(word)
    return Boolean(ret.value) && !ret.isSubstring
  }

  /** Returns the segments found when traversing the trie for the given word. */
  segments(word: string) {
    const ret = this.traverse(word)
    return !ret.segments.length ? null : ret.segments
  }

  /** Returns the longest/last segment for a word. */
  lastSegment(word: string) {
    const segments = this.segments(word)
    return !segments ? null : segments[segments.length - 1]
  }

  /** Removes a word from the trie. */
  remove(word: string) {
    this.tryDelete(word, -1, this.head)
  }

  private traverse(word: string): TraverseResult<T>
  private traverse(word: string, add: true): TrieMap<T>
  private traverse(word: string, add = false): TrieMap<T> | TraverseResult<T> {
    let current = this.head

    const segments: T[] = []

    for (let i = 0; i < word.length; i++) {
      if (current.has(END_WORD_SYMBOL)) {
        segments.push(current.get(END_WORD_SYMBOL) as T)
      }
      if (!current.get(word[i])) {
        if (add) {
          current.set(word[i], new Map())
        } else {
          return {
            value: current.get(END_WORD_SYMBOL)! as T,
            word: word.substring(0, i),
            segments,
            isSubstring: true
          }
        }
      }
      // keep traversing
      current = current.get(word[i])! as TrieMap<T>
    }

    if (add) {
      return current
    }

    // found the word
    return {
      value: current.get(END_WORD_SYMBOL)! as T,
      word: word,
      segments,
      isSubstring: false
    }
  }

  private tryDelete(word = "", index = 0, node: TrieMap<T> | null = null) {
    if (index >= word.length) {
      throw new Error("Bad index to check for deletion.")
    }
    if (node === null) {
      throw new Error(`Bad Node at ${index} for ${word}`)
    }

    const currentNode = node

    if (index === word.length - 1) {
      return currentNode.delete(END_WORD_SYMBOL) && node.size === 0
    }

    const newIndex = word[index + 1]
    if (this.tryDelete(word, index + 1, node.get(newIndex) as TrieMap<T>)) {
      return currentNode.delete(END_WORD_SYMBOL) && node.size === 0
    }

    return false
  }
}

interface TraverseResult<T> {
  value: undefined | T
  word: string
  segments: T[]
  isSubstring: boolean
}
