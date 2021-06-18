import { re } from "../util"

export class ConvTable {
  declare table: { pattern: RegExp; replacement: string }[]

  constructor(pairs: [string, string][]) {
    this.table = pairs.map(([pattern, replacement]) => {
      const cleanedPattern = pattern
        .replace(/^_/, "^")
        .replace(/_$/, "$")
        .replaceAll("_", "")
      return {
        pattern: re`/${cleanedPattern}/y`,
        replacement: replacement.replaceAll("_", " ")
      }
    })
  }

  match(word: string) {
    // * for each position in word
    // * ...find any matching rules
    // * ...chose the one with longest pattern
    // * ...apply it, and shift to position after its applied
    let pos = 0
    let res = ""
    while (pos < word.length) {
      let replacement: string | null = null
      let len = 0
      let max = 0

      for (const pair of this.table) {
        pair.pattern.lastIndex = pos
        if (pair.pattern.source.length > max && pair.pattern.test(word)) {
          pair.pattern.lastIndex = pos
          len = pair.pattern.exec(word)![0].length
          max = pair.pattern.source.length
          replacement = pair.replacement
        }
      }

      if (replacement) {
        res += replacement
        pos += len
      } else {
        res += word[pos]
        pos += 1
      }
    }

    return res
  }
}
