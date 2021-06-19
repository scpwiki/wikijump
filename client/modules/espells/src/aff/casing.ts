import { CapType } from "../constants"
import {
  isLowercased,
  isTitlecased,
  isUppercased,
  lowercase,
  replaceRange,
  titlecase,
  uppercase
} from "../util"

export class Casing {
  guess(word: string) {
    if (isLowercased(word)) return CapType.NO
    if (isUppercased(word)) return CapType.ALL
    if (isTitlecased(word)) return CapType.INIT
    if (isUppercased(word[0])) return CapType.HUHINIT
    return CapType.HUH
  }

  lower(word: string) {
    return [lowercase(word)]
  }

  upper(word: string) {
    return uppercase(word)
  }

  capitalize(word: string) {
    return [titlecase(word)]
  }

  lowerfirst(word: string) {
    return [replaceRange(word, 0, 1, lowercase(word[0]))]
  }

  variants(word: string): [CapType, ...string[]] {
    const captype = this.guess(word)
    // prettier-ignore
    switch (captype) {
      case CapType.HUH:
      case CapType.NO:      return [captype, word]
      case CapType.INIT:    return [captype, ...this.lower(word)]
      case CapType.HUHINIT: return [captype, ...this.lowerfirst(word)]
      case CapType.ALL:     return [captype, ...this.lower(word), ...this.capitalize(word)]
    }
  }

  corrections(word: string): [CapType, ...string[]] {
    const captype = this.guess(word)
    // prettier-ignore
    switch (captype) {
      case CapType.NO:      return [captype, word]
      case CapType.INIT:    return [captype, ...this.lower(word)]
      case CapType.HUHINIT: return [captype, ...this.lowerfirst(word), ...this.lower(word), ...this.capitalize(word)]
      case CapType.HUH:     return [captype, ...this.lower(word)]
      case CapType.ALL:     return [captype, ...this.lower(word), ...this.capitalize(word)]
    }
  }

  coerce(word: string, cap: CapType) {
    // prettier-ignore
    switch(cap) {
      case CapType.INIT:
      case CapType.HUHINIT: return this.upper(word[0]) + word.slice(1)
      case CapType.ALL: return this.upper(word)
      default: return word
    }
  }
}

export class TurkicCasing extends Casing {
  private replaceMapping(word: string, dir: -1 | 1) {
    if (!/İiIı/u.test(word)) return word
    return dir < 0
      ? word.replaceAll("İ", "i").replaceAll("I", "ı")
      : word.replaceAll("i", "İ").replaceAll("ı", "I")
  }

  override lower(word: string) {
    return super.lower(this.replaceMapping(word, -1))
  }

  override upper(word: string) {
    return super.upper(this.replaceMapping(word, 1))
  }
}

export class GermanCasing extends Casing {
  private sharpVariants(word: string, start = 0): string[] {
    const pos = word.indexOf("ss", start)
    if (pos === -1) return []
    const replaced = replaceRange(word, pos, pos + 2, "ß")
    return [
      replaced,
      ...this.sharpVariants(replaced, pos + 1),
      ...this.sharpVariants(word, pos + 2)
    ]
  }

  override lower(word: string) {
    const lowered = super.lower(word)[0]
    if (word.includes("SS")) {
      return [...this.sharpVariants(lowered), lowered]
    } else {
      return [lowered]
    }
  }

  override guess(word: string) {
    if (word.includes("ß") && super.guess(word.replaceAll("ß", "")) === CapType.ALL) {
      return CapType.ALL
    }
    return super.guess(word)
  }
}
