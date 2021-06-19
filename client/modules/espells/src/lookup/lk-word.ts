import type { CapType, CompoundPos } from "../constants"

/**
 * A word (a string) wrapped with metadata. Can be iterated, and will
 * coerce itself to a string.
 */
export class LKWord {
  constructor(
    /** The word itself. */
    public word: string,
    /** The capitalization type of the word. */
    public type: CapType,
    /** The position of the word in a compound, if any. */
    public pos?: CompoundPos
  ) {}

  /**
   * Reuses this instance's metadata on a new word.
   *
   * @param word - The new word string to use.
   */
  to(word: string) {
    return new LKWord(word, this.type, this.pos)
  }

  /**
   * Returns a new {@link LKWord} from a section of this word.
   *
   * @param from - The starting index of the section. Can be negative.
   * @param to - The ending index of the section.
   */
  slice(from?: number, to?: number) {
    return this.to(this.word.slice(from, to))
  }

  /**
   * Executes an ordinary text replacement operation on this word and
   * returns a new instance from the result.
   *
   * @param pat - The object that will search for matches in the word.
   * @param repl - The replacement string for the found match.
   */
  replace(pat: { [Symbol.replace](s: string, r: string): string }, repl = "") {
    return this.to(this.word.replace(pat, repl))
  }

  /**
   * Executes an ordinary text replacement operation on this word and
   * returns a new instance from the result. Replaces all matches, rather
   * than just the first one found.
   *
   * @param pat - The global `RegExp` or string to match with.
   * @param repl - The replacement string for the found match.
   */
  replaceAll(pat: string | RegExp, repl = "") {
    return this.to(this.word.replaceAll(pat, repl))
  }

  /**
   * Adds (concatenates) a string (or another {@link LKWord}) to this word
   * and returns a new instance from the result.
   *
   * @param str - The string or {@link LKWord} to add.
   */
  add(str: string | LKWord) {
    if (str instanceof LKWord) str = str.word
    return this.to(this.word + str)
  }

  /**
   * Gets the character at the specified index. Accepts negative numbers.
   *
   * @param n - The index of the desired character. Can be negative.
   */
  at(n: number) {
    if (n < 0) return this.word[this.word.length - n]
    return this.word[n]
  }

  /** The length of the word. */
  get length() {
    return this.word.length
  }

  [Symbol.toStringTag]() {
    return this.word
  }

  *[Symbol.iterator]() {
    yield* this.word
  }

  [Symbol.toPrimitive]() {
    return this.word
  }
}
