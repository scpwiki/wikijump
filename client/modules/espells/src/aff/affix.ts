import iterate from "iterare"
import { CONSTANTS as C } from "../constants"
import { re } from "../util"
import type { Aff, Flag, Flags } from "./index"

/** Base class for the {@link Prefix}/{@link Suffix} classes. Won't work by itself. */
export abstract class Affix {
  /** The {@link Flag} that denotes the "name" for this affix. */
  declare flag: Flag

  /**
   * If true, this affix is allowed to have the opposite form of this
   * affix, i.e. if a {@link Suffix} would be allowed to have a
   * {@link Prefix}, and vice versa.
   */
  declare crossproduct: boolean

  /**
   * What is "stripped" from the stem when this affix is applied. e.g. this
   * would be `"e"` for the suffix `"or"` (as in `create`, `creator`)
   */
  declare strip: string

  /**
   * What to add when this affix is applied. e.g. this would be `"or"` in
   * the transformation needed for `create`, `creator`.
   */
  declare add: string

  /** {@link Flags} this affix has been marked with. */
  declare flags: Flags

  /**
   * `RegExp` tested against a word to determine if this affix is relevant
   * to said word. This doesn't check if the word *has* this affix, just if
   * it *can* have this affix.
   */
  declare conditionRegex: RegExp

  /** `RegExp` tested against a word to determine if said word has this affix. */
  declare lookupRegex: RegExp

  /**
   * `RegExp` tested against a word to match the specific section of said
   * word that would need to be stripped to apply this affix.
   */
  declare replaceRegex: RegExp

  /**
   * @param flag - {@link Flag} that this affix will be labled with.
   * @param crossproduct - Sets the {@link Affix.crossproduct} state, with
   *   the string "Y" meaning `true` and everything else meaning `false`.
   * @param strip - What this affix should strip from a word to be applied.
   *   If given as the string "0", that means not to strip anything.
   * @param add - What to add to a word when this affix is applied. If
   *   given as the string "0", that means to add nothing.
   * @param aff - The {@link Aff} data to use when parsing flags.
   */
  constructor(flag: Flag, crossproduct: string, strip: string, add: string, aff: Aff) {
    let flags: string
    ;[add, flags] = add.split("/")

    if (aff.IGNORE) {
      for (const ch of aff.IGNORE) {
        add = add.replaceAll(ch, "")
      }
    }

    this.flag = flag
    this.crossproduct = crossproduct === "Y"
    this.strip = strip === "0" ? "" : strip
    this.add = add === "0" ? "" : add
    this.flags = flags ? aff.parseFlags(flags) : new Set()
  }

  /**
   * Determines if a word matches the conditions of this affix.
   *
   * @param word - The word to check against this affix's conditions.
   */
  relevant(word: string) {
    return this.conditionRegex.test(word)
  }

  /**
   * Determines if a word already has this affix applied to it.
   *
   * @param word - The word to check for if this affix is already present.
   */
  on(word: string) {
    return this.lookupRegex.test(word)
  }

  /**
   * Applies this affix to a word, returning the word as a transformed string.
   *
   * @param word - The word to apply the transformation to.
   */
  apply(word: string) {
    return word.replace(this.replaceRegex, this.strip)
  }

  /**
   * Determines if this affix has the given flag.
   *
   * @param flag - The flag to check for. Can be undefined, which will return false.
   */
  has(flag?: Flag) {
    if (flag === undefined) return false
    return this.flags.has(flag)
  }

  /**
   * Determines if this affix is compatible with a set of flags, meaning
   * that the affix's flags are present in the flag set given.
   *
   * @param flags - The flags to check against. Every flag this affix has
   *   must be in this argument.
   * @param forbidden - An optional set of flags which has the inverse
   *   effect, meaning that this affix's flags *cannot* be found in this set.
   */
  compatible(flags: Flags, forbidden?: Flags) {
    for (const flag of this.flags) {
      if (!flags.has(flag) || forbidden?.has(flag)) return false
    }
    return true
  }
}

/** An {@link Affix} that is applied to or found at the beginning of a stem. */
export class Prefix extends Affix {
  /**
   * @param flag - {@link Flag} that this affix will be labled with.
   * @param crossproduct - Sets the {@link Affix.crossproduct} state, with
   *   the string "Y" meaning `true` and everything else meaning `false`.
   * @param strip - What this affix should strip from a word to be applied.
   *   If given as the string "0", that means not to strip anything.
   * @param add - What to add to a word when this affix is applied. If
   *   given as the string "0", that means to add nothing.
   * @param condition - A `RegExp` like pattern to check against a word to
   *   see if this affix is relevant.
   * @param aff - The {@link Aff} data to use when parsing flags.
   */
  constructor(
    flag: string,
    crossproduct: string,
    strip: string,
    add: string,
    condition: string,
    aff: Aff
  ) {
    super(flag, crossproduct, strip, add, aff)

    let parts = iterate(condition.matchAll(C.SPLIT_CONDITION_REGEX))
      .map(part => part.slice(1))
      .flatten()
      .toArray()

    if (parts.length && this.strip) parts = parts.slice(this.strip.length)

    let cond = ""

    if (parts.length && !(parts.length === 1 && parts[0] === ".")) {
      cond = `(?=${parts.join("")})`.replaceAll("-", "\\-")
    }

    this.conditionRegex = re`/^${condition.replaceAll("-", "\\-")}/`
    this.lookupRegex = re`/^${this.add}${cond}/`
    this.replaceRegex = re`/^${this.add}/`
  }
}

/** An {@link Affix} that is applied to or can be found at the end of a stem. */
export class Suffix extends Affix {
  /**
   * @param flag - {@link Flag} that this affix will be labled with.
   * @param crossproduct - Sets the {@link Affix.crossproduct} state, with
   *   the string "Y" meaning `true` and everything else meaning `false`.
   * @param strip - What this affix should strip from a word to be applied.
   *   If given as the string "0", that means not to strip anything.
   * @param add - What to add to a word when this affix is applied. If
   *   given as the string "0", that means to add nothing.
   * @param condition - A `RegExp` like pattern to check against a word to
   *   see if this affix is relevant.
   * @param aff - The {@link Aff} data to use when parsing flags.
   */
  constructor(
    flag: string,
    crossproduct: string,
    strip: string,
    add: string,
    condition: string,
    aff: Aff
  ) {
    super(flag, crossproduct, strip, add, aff)

    let parts = iterate(condition.matchAll(C.SPLIT_CONDITION_REGEX))
      .map(part => part.slice(1))
      .flatten()
      .toArray()

    let cond = ""

    if (parts.length && !(parts.length === 1 && parts[0] === ".")) {
      if (this.strip) parts = parts.slice(0, -this.strip.length)
      cond = `(${parts.join("")})`.replaceAll("-", "\\-")
    }

    this.conditionRegex = re`/${condition.replaceAll("-", "\\-")}$/`
    this.lookupRegex = re`/${cond}${this.add}$/`
    this.replaceRegex = re`/${this.add}$/`
  }
}
