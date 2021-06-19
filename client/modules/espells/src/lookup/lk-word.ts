import type { CapType } from "../aff/casing"
import type { CompoundPos } from "./forms"

export class LKWord {
  constructor(
    public word: string,
    public captype: CapType,
    public compoundpos?: CompoundPos
  ) {}

  to(word: string) {
    return new LKWord(word, this.captype)
  }

  slice(from?: number, to?: number) {
    return this.to(this.word.slice(from, to))
  }

  replace(pat: { [Symbol.replace](s: string, r: string): string }, repl = "") {
    return this.to(this.word.replace(pat, repl))
  }

  replaceAll(pat: string | RegExp, repl = "") {
    return this.to(this.word.replaceAll(pat, repl))
  }

  add(str: string | LKWord) {
    if (str instanceof LKWord) str = str.word
    return this.to(this.word + str)
  }

  at(n: number) {
    if (n < 0) return this.word[this.word.length - n]
    return this.word[n]
  }

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
