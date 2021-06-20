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

/**
 * A class containing casing-related algorithms for a language. This is a
 * class so that it may be extended by subclasses, e.g. German which has
 * special handling of the "sharp S" character.
 */
export class Casing {
  /**
   * Guess a word's capitalization.
   *
   * @param word - The word to check.
   */
  guess(word: string) {
    if (isLowercased(word)) return CapType.NO
    if (isUppercased(word)) return CapType.ALL
    if (isTitlecased(word)) return CapType.INIT
    if (isUppercased(word[0])) return CapType.HUHINIT
    return CapType.HUH
  }

  /**
   * Lowercases a word. This returns an array of potential variations of
   * lowercasing. This is done because certain languages (really, just
   * German) have multiple ways of representing the same word in lowercase.
   *
   * @param word - The word to lowercase.
   */
  lower(word: string) {
    return [lowercase(word)]
  }

  /**
   * Uppercases a word.
   *
   * @param word - The word to uppercase.
   */
  upper(word: string) {
    return uppercase(word)
  }

  /**
   * Titlecases a word.
   *
   * @param word - The word to titlecase.
   */
  capitalize(word: string) {
    return [titlecase(word)]
  }

  /**
   * Lowercases just the first letter of a word. Returns an array for the
   * same reasons as {@link Casing.lower}.
   *
   * @param word - The word to lowercase the first letter of.
   */
  lowerfirst(word: string) {
    return [replaceRange(word, 0, 1, lowercase(word[0]))]
  }

  /**
   * Returns a list of potential ways a word may have been cased in the
   * dictionary, if we consider it to be spelled correctly but potentially
   * *cased* incorrectly. The first item in the returned list will be the
   * {@link CapType} of the given word.
   *
   * @param word - The word to return the variations of.
   */
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

  /**
   * Returns a list of potential ways a word might have been cased if it
   * seems to be a misspelling. The first item in the list will be the
   * {@link CapType} of the given word.
   *
   * @param word - The word to return the corrections of.
   */
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

  /**
   * Used by the suggestion algorithm. If a misspelled word has been found
   * in the dictionary, but is inconsistent with the casing of the word
   * found in the dictionary, this method will be used to attempt to
   * "coerce" the dictionary word to be cased like how the original misspelling was.
   *
   * @param word - The word to coerce.
   * @param cap - The {@link CapType} to coerce the word to.
   */
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

/**
 * A special subclass of {@link Casing} for Turkic languages, which have
 * unique rules for lowercasing and uppercasing the characters `İ,I`, `i,ı`.
 */
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

/**
 * A special subclass of {@link Casing} for German, which has a unique rule
 * where `"SS"` can be lowercased as both `"ss"` *and* `“ß”`.
 */
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
