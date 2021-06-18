import iterate from "iterare"
import type { Aff } from "../aff"
import { CapType } from "../aff/casing"
import type { Reader } from "../reader"
import { Word } from "./word"

const SKIP_REGEX = /^\d+(\s+|$)|^\/|^\t|^\s*$/

export class Dic {
  words: Set<Word> = new Set()
  private index: Map<string, Set<Word> | Word> = new Map()
  private lowercaseIndex: Map<string, Set<Word>> = new Map()

  private declare aff: Aff

  constructor(reader: Reader, aff: Aff) {
    this.aff = aff
    this.addDictionary(reader)
  }

  private addToIndex(index: Map<string, Set<Word> | Word>, stem: string, word: Word) {
    const curr = index.get(stem)
    if (curr) {
      if (curr instanceof Set) curr.add(word)
      else index.set(stem, new Set([curr, word]))
    } else {
      index.set(stem, word)
    }
  }

  private getFromindex(index: Map<string, Set<Word> | Word>, stem: string): Set<Word> {
    if (!index.has(stem)) return new Set()
    const words = index.get(stem)!
    // have to clone to prevent mutating the dictionary
    return words instanceof Set ? new Set([...words]) : new Set([words])
  }

  addDictionary(reader: Reader) {
    do {
      if (reader.done) break
      if (SKIP_REGEX.test(reader.line)) continue
      this.add(new Word(reader.line, this.aff))
    } while (reader.next())
  }

  add(word: Word) {
    const stem = word.stem
    this.words.add(word)
    this.addToIndex(this.index, stem, word)
    if (this.aff.casing.guess(stem) !== CapType.NO) {
      const lowers = this.aff.casing.lower(stem)
      lowers.forEach(lowered => this.addToIndex(this.lowercaseIndex, lowered, word))
    }
  }

  remove(stem: string) {
    const words = this.homonyms(stem, true)
    if (!words) return
    for (const word of words) {
      this.words.delete(word)
    }
    this.index.delete(stem)
    const lowers = this.aff.casing.lower(stem)
    lowers.forEach(lowered => this.lowercaseIndex.delete(lowered))
  }

  homonyms(stem: string, ignorecase = false) {
    const words = this.getFromindex(this.index, stem)
    if (ignorecase) {
      const lowers = this.aff.casing.lower(stem)
      lowers.forEach(lowered =>
        iterate(this.getFromindex(this.lowercaseIndex, lowered)).forEach(word =>
          words.add(word)
        )
      )
    }
    return words
  }

  hasFlag(stem: string, flag: string, all = false) {
    const homonyms = this.homonyms(stem)
    if (!homonyms) return false
    for (const word of homonyms) {
      const flagged = word.flags?.has(flag)
      if (all && !flagged) return false
      if (!all && flagged) return true
    }
    return false
  }
}
