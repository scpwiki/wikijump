import type { Aff, Flags } from "../aff"
import type { CapType } from "../aff/casing"
import { RepPattern } from "../aff/rep-pattern"

// 1. stem, 2. flags, 3. data (not split)
const SPLIT_WORD_REGEX = /^(.+?)(?:\/([\S\t]*?))?(?:(?:\s(?=.*?:.))(.+))?$/
// 1. key, 2. value
const SPLIT_DATA_REGEX = /(\S+):(\S+)/

export class Word {
  declare stem: string
  declare capType: CapType

  // using optionals here to save on memory
  declare flags?: Flags
  declare data?: Map<string, Set<string>>
  declare altSpellings?: Set<string>

  constructor(line: string, aff: Aff) {
    const match = SPLIT_WORD_REGEX.exec(line)
    if (!match) throw new SyntaxError(`Invalid line in dictionary '${line}'`)
    let [, stem, flags, data] = match

    stem = stem.replaceAll("\\/", "/")

    this.stem = stem
    this.capType = aff.casing.guess(stem)

    if (flags) this.flags = aff.parseFlags(flags)

    if (data) {
      for (const keyvalue of data.split(/\s+/)) {
        const match = SPLIT_DATA_REGEX.exec(keyvalue)

        // key:value pair
        if (match) {
          const [, key, value] = match

          this.data ??= new Map()
          const set = this.data.get(value) ?? new Set()
          this.data.set(key, set.add(value))

          // ph: misspellings
          if (key === "ph") {
            // pretty ph:prit* -> rep(prit, prett)
            if (value.endsWith("*")) {
              aff.REP.add(new RepPattern(value.slice(0, -2), stem.slice(0, -1)))
            }
            // happy ph:hepi->happi -> rep(hepi, happi)
            else if (value.includes("->")) {
              const [from, to] = value.split("->")
              aff.REP.add(new RepPattern(from, to))
            }
            // wednesday ph:wensday -> rep(wensday, wednesday)
            // and altSpelling added for ngram suggestions
            else {
              aff.REP.add(new RepPattern(value, stem))
              this.altSpellings ??= new Set()
              this.altSpellings.add(value)
            }
          }
        }
        // morphology alias
        else if (/^\d+$/.test(keyvalue) && aff.AM[parseInt(keyvalue) - 1]) {
          for (const str in aff.AM[parseInt(keyvalue) - 1]) {
            this.data ??= new Map()
            const set = this.data.get(keyvalue) ?? new Set()
            this.data.set(keyvalue, set.add(str))
          }
        }
      }
    }
  }
}
