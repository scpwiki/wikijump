import iterate from "iterare"
import type { Aff } from "../aff"
import { Prefix, Suffix } from "../aff/affix"
import { CapType } from "../aff/casing"
import type { CompoundRule } from "../aff/compound-rule"
import type { Dic } from "../dic"
import type { Word } from "../dic/word"
import { replchars } from "../permutations"
import { any, isUppercased, lowercase, reverse } from "../util"
import { AffixForm, CompoundForm, CompoundPos } from "./forms"

const NUMBER_REGEX = /^\d+(\.\d+)?$/

export interface LookupResult {
  correct: boolean
  forbidden: boolean
  warn: boolean
}

/** Class that facilitaties lookups for a spellchecker. */
export class Lookup {
  /** Spellchecker's affix data. */
  declare aff: Aff

  /** Spellchecker's dictionary data. */
  declare dic: Dic

  /**
   * @param aff - The affix data to use.
   * @param dic - The dictionary data to use.
   */
  constructor(aff: Aff, dic: Dic) {
    this.aff = aff
    this.dic = dic
  }

  /**
   * Checks if a word is spelled correctly.
   *
   * @param word - The word to check.
   * @param caps - If true, checking will be case sensitive. Defaults to true.
   * @param allowNoSuggest - If false, words which are in the dictionary,
   *   but are flagged with the `NOSUGGEST` flag (if provided), will not be
   *   considered correct. Defaults to true.
   */
  test(word: string, caps = true, allowNoSuggest = true): LookupResult {
    let forbidden = this.isForbidden(word)
    let warn = this.isWarn(word)

    if (forbidden) return { correct: false, forbidden, warn }

    if (this.aff.ICONV) word = this.aff.ICONV.match(word)

    if (this.aff.IGNORE) {
      for (const ch of this.aff.IGNORE) {
        word = word.replaceAll(ch, "")
      }
    }

    if (NUMBER_REGEX.test(word)) return { correct: true, forbidden, warn }

    for (const word2 of iterate(this.breakWord(word)).flatten()) {
      if (!any(this.goodForms(word2, caps, allowNoSuggest))) {
        return { correct: false, forbidden, warn }
      }
    }

    return { correct: true, forbidden, warn }
  }

  private isWarn(word: string) {
    return this.dic.hasFlag(word, this.aff.WARN, true)
  }

  private isForbidden(word: string) {
    return (
      this.dic.hasFlag(word, this.aff.FORBIDDENWORD, true) ||
      (this.aff.FORBIDWARN && this.dic.hasFlag(word, this.aff.WARN, true))
    )
  }

  /**
   * Yields permutations of a word split up (with whitespace) using the
   * `BREAK` rules given by the spellchecker's {@link Aff} data.
   *
   * @param text - The word/text to split.
   * @param depth - The current depth of the check. Used by this function
   *   when calling itself recursively. There isn't any need to set it yourself.
   */
  *breakWord(text: string, depth = 0): Generator<string[]> {
    if (depth > 10) return
    yield [text]
    for (const pattern of this.aff.BREAK) {
      for (const m of text.matchAll(pattern)) {
        const start = text.slice(0, m.index!)
        const rest = text.slice(0, m.index! + m[0].length)
        for (const breaking of this.breakWord(rest, depth + 1)) {
          yield [start, ...breaking]
        }
      }
    }
  }

  // -- CHECKING WORDS

  /**
   * Yields combinations of stems and affixes for a word, specifically
   * yielding instances of {@link AffixForm}. If this function does actually
   * yield a form, that means that it can be considered as spelled correctly.
   *
   * @param word - The word to check.
   * @param caps - Case sensitive if true. Defaults to true.
   * @param allowNoSuggest - Yields stems flagged as `NOSUGGEST` if true.
   *   Defaults to true.
   * @param affixForms - If false, {@link AffixForm} instances won't be
   *   yielded. Defaults to true.
   * @param compoundForms - If false, {@link CompoundForm} instances won't
   *   be yielded. Defaults to true.
   */
  *goodForms(
    word: string,
    caps = true,
    allowNoSuggest = true,
    affixForms = true,
    compoundForms = true
  ) {
    let captype: CapType, variants: string[]
    if (caps) {
      ;[captype, ...variants] = this.aff.casing.variants(word)
    } else {
      captype = this.aff.casing.guess(word)
      variants = [word]
    }

    for (const variant of variants) {
      if (affixForms) {
        for (const form of this.affixForms(variant, captype, allowNoSuggest)) {
          if (
            this.aff.CHECKSHARPS &&
            this.aff.KEEPCASE &&
            form.inDictionary?.stem.includes("ß") &&
            form.flags.has(this.aff.KEEPCASE) &&
            captype === CapType.ALL &&
            word.includes("ß")
          ) {
            continue
          }
          yield form
        }
      }
      if (compoundForms) {
        yield* this.compoundForms(word, captype, allowNoSuggest)
      }
    }
  }

  isGoodForm(
    form: AffixForm,
    compoundpos: CompoundPos | null,
    captype: CapType,
    allowNoSuggest = true
  ) {
    if (!form.inDictionary) return false

    const aff = this.aff

    const rootFlags = form.inDictionary.flags ?? new Set()
    const allFlags = form.flags

    if (!allowNoSuggest && aff.NOSUGGEST && rootFlags.has(aff.NOSUGGEST)) {
      return false
    }

    if (
      captype !== form.inDictionary.capType &&
      aff.KEEPCASE &&
      rootFlags.has(aff.KEEPCASE)
    ) {
      if (!(aff.CHECKSHARPS && form.inDictionary.stem.includes("ß"))) {
        return false
      }
    }

    if (aff.NEEDAFFIX) {
      if (rootFlags.has(aff.NEEDAFFIX) && !form.hasAffixes) {
        return false
      }
      if (
        form.hasAffixes &&
        form.affixes().every(affix => affix.flags.has(aff.NEEDAFFIX!))
      ) {
        return false
      }
    }

    if (form.prefix && !allFlags.has(form.prefix.flag)) {
      return false
    }

    if (form.suffix && !allFlags.has(form.suffix.flag)) {
      return false
    }

    if (aff.CIRCUMFIX) {
      const suffixHas = Boolean(form.suffix?.flags.has(aff.CIRCUMFIX))
      const prefixHas = Boolean(form.prefix?.flags.has(aff.CIRCUMFIX))
      if (suffixHas !== prefixHas) return false
    }

    if (compoundpos === null) return !allFlags.has(aff.ONLYINCOMPOUND!)

    if (aff.COMPOUNDFLAG && !allFlags.has(aff.COMPOUNDFLAG)) {
      return true
    }

    // prettier-ignore
    switch(compoundpos) {
      case CompoundPos.BEGIN: return allFlags.has(aff.COMPOUNDBEGIN!)
      case CompoundPos.MIDDLE: return allFlags.has(aff.COMPOUNDMIDDLE!)
      case CompoundPos.END: return allFlags.has(aff.COMPOUNDEND!)
    }
  }

  // -- DECOMPOSING WORDS

  isGoodAffix(
    affix: Prefix | Suffix,
    word: string,
    requiredFlags: Set<string>,
    forbiddenFlags: Set<string>,
    crossproduct = false
  ) {
    if (affix instanceof Suffix) {
      if (!(crossproduct || affix.crossproduct)) return false
    }

    for (const flag of affix.flags) {
      if (forbiddenFlags.has(flag) || !requiredFlags.has(flag)) return false
    }

    return affix.lookupRegex.test(word)
  }

  *desuffix(
    word: string,
    requiredFlags: Set<string>,
    forbiddenFlags: Set<string>,
    nested = false,
    crossproduct = false
  ): Generator<AffixForm> {
    const segments = this.aff.suffixesIndex.segments(reverse(word))

    if (segments) {
      const possibleSuffixes = iterate(segments)
        .flatten()
        .filter(suffix =>
          this.isGoodAffix(suffix, word, requiredFlags, forbiddenFlags, crossproduct)
        )

      for (const suffix of possibleSuffixes) {
        const stem = word.replace(suffix.replaceRegex, suffix.strip)
        yield new AffixForm(word, stem, { suffix })
        if (!nested) {
          for (const form2 of this.desuffix(
            stem,
            iterate(suffix.flags).concat(requiredFlags).toSet(),
            forbiddenFlags,
            true,
            crossproduct
          )) {
            yield form2.replace({ text: word, suffix2: suffix })
          }
        }
      }
    }
  }

  *deprefix(
    word: string,
    requiredFlags: Set<string>,
    forbiddenFlags: Set<string>,
    nested = false
  ): Generator<AffixForm> {
    const segments = this.aff.prefixesIndex.segments(word)

    if (segments) {
      const possiblePrefixes = iterate(segments)
        .flatten()
        .filter(prefix => this.isGoodAffix(prefix, word, requiredFlags, forbiddenFlags))

      for (const prefix of possiblePrefixes) {
        const stem = word.replace(prefix.replaceRegex, prefix.strip)
        yield new AffixForm(word, stem, { prefix })
        if (!nested && this.aff.COMPLEXPREFIXES) {
          for (const form2 of this.deprefix(
            stem,
            iterate(prefix.flags).concat(requiredFlags).toSet(),
            forbiddenFlags,
            true
          )) {
            yield form2.replace({ text: word, prefix2: prefix })
          }
        }
      }
    }
  }

  // -- AFFIX FORMS

  *affixForms(
    word: string,
    captype: CapType,
    allowNoSuggest = true,
    prefixFlags: Set<string> = new Set(),
    suffixFlags: Set<string> = new Set(),
    forbiddenFlags: Set<string> = new Set(),
    compoundpos: CompoundPos | null = null,
    withForbidden = false
  ) {
    const isGood = (form: AffixForm) =>
      this.isGoodForm(form, compoundpos, captype, allowNoSuggest)

    for (const form of this.produceAffixForms(
      word,
      prefixFlags,
      suffixFlags,
      forbiddenFlags,
      compoundpos
    )) {
      let found = false

      const homonyms = this.dic.homonyms(form.stem)

      if (homonyms.size) {
        if (
          !withForbidden &&
          this.aff.FORBIDDENWORD &&
          (compoundpos !== null || form.hasAffixes) &&
          iterate(homonyms).some(({ flags }) => !!flags?.has(this.aff.FORBIDDENWORD!))
        ) {
          return
        }

        for (const homonym of homonyms) {
          const candidate = form.replace({ inDictionary: homonym })
          if (isGood(candidate)) {
            found = true
            yield candidate
          }
        }
      }

      if (
        compoundpos === CompoundPos.BEGIN &&
        this.aff.FORCEUCASE &&
        captype === CapType.INIT
      ) {
        for (const homonym of this.dic.homonyms(lowercase(form.stem))) {
          const candidate = form.replace({ inDictionary: homonym })
          if (isGood(candidate)) {
            found = true
            yield candidate
          }
        }
      }

      if (found || compoundpos !== null || captype !== CapType.ALL) continue

      if (this.aff.casing.guess(word) === CapType.NO) {
        for (const homonym of this.dic.homonyms(form.stem, true)) {
          const candidate = form.replace({ inDictionary: homonym })
          if (isGood(candidate)) {
            yield candidate
          }
        }
      }
    }
  }

  *produceAffixForms(
    word: string,
    prefixFlags: Set<string>,
    suffixFlags: Set<string>,
    forbiddenFlags: Set<string>,
    compoundpos: CompoundPos | null = null
  ) {
    yield new AffixForm(word, word)

    const suffixAllowed =
      compoundpos === null || compoundpos === CompoundPos.END || suffixFlags.size
    const prefixAllowed =
      compoundpos === null || compoundpos === CompoundPos.BEGIN || prefixFlags.size

    if (suffixAllowed) {
      yield* this.desuffix(word, suffixFlags, forbiddenFlags)
    }
    if (prefixAllowed) {
      for (const form of this.deprefix(word, prefixFlags, forbiddenFlags)) {
        yield form
        if (suffixAllowed && form?.prefix?.crossproduct) {
          for (const form2 of this.desuffix(
            form.stem,
            suffixFlags,
            forbiddenFlags,
            false,
            true
          )) {
            yield form2.replace({ text: form.text, prefix: form.prefix })
          }
        }
      }
    }
  }

  // -- COMPOUND FORMS

  *compoundForms(word: string, captype: CapType, allowNoSuggest: boolean) {
    if (this.aff.FORBIDDENWORD) {
      // TODO: lol fix this
      for (const candidate of this.affixForms(
        word,
        captype,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        true
      )) {
        if (candidate.flags.has(this.aff.FORBIDDENWORD)) return
      }
    }

    if (this.aff.COMPOUNDBEGIN || this.aff.COMPOUNDFLAG) {
      for (const compound of this.compoundsByFlags(
        word,
        captype,
        undefined,
        allowNoSuggest
      )) {
        if (!this.isBadCompound(compound, captype)) {
          yield compound
        }
      }
    }

    if (this.aff.COMPOUNDRULE) {
      for (const compound of this.compoundsByRules(
        word,
        undefined,
        undefined,
        allowNoSuggest
      )) {
        if (!this.isBadCompound(compound, captype)) {
          yield compound
        }
      }
    }
  }

  *compoundsByFlags(
    wordRest: string,
    captype: CapType,
    depth = 0,
    allowNoSuggest = true
  ): Generator<CompoundForm> {
    const aff = this.aff

    const forbiddenFlags = new Set(aff.COMPOUNDFORBIDFLAG ? [aff.COMPOUNDFORBIDFLAG] : [])
    const permitFlags = new Set(aff.COMPOUNDPERMITFLAG ? [aff.COMPOUNDPERMITFLAG] : [])

    if (depth) {
      for (const form of this.affixForms(
        wordRest,
        captype,
        allowNoSuggest,
        permitFlags,
        undefined,
        forbiddenFlags
      )) {
        yield [form]
      }
    }

    if (
      wordRest.length < aff.COMPOUNDMIN * 2 ||
      (aff.COMPOUNDWORDMAX && depth >= aff.COMPOUNDWORDMAX)
    ) {
      return
    }

    const compoundpos = depth ? CompoundPos.MIDDLE : CompoundPos.BEGIN
    const prefixFlags: Set<string> =
      compoundpos === CompoundPos.BEGIN ? new Set() : permitFlags

    for (let pos = aff.COMPOUNDMIN; pos < wordRest.length - aff.COMPOUNDMIN + 1; pos++) {
      const beg = wordRest.slice(0, pos)
      const rest = wordRest.slice(pos)

      for (const form of this.affixForms(
        beg,
        captype,
        allowNoSuggest,
        prefixFlags,
        permitFlags,
        forbiddenFlags,
        compoundpos
      )) {
        for (const partial of this.compoundsByFlags(
          rest,
          captype,
          depth + 1,
          allowNoSuggest
        )) {
          yield [form, ...partial]
        }
      }

      if (aff.SIMPLIFIEDTRIPLE && beg[-1] === rest[0]) {
        for (const form of this.affixForms(
          beg + beg[-1],
          captype,
          allowNoSuggest,
          prefixFlags,
          permitFlags,
          forbiddenFlags,
          compoundpos
        )) {
          for (const partial of this.compoundsByFlags(
            rest,
            captype,
            depth + 1,
            allowNoSuggest
          )) {
            yield [form.replace({ text: beg }), ...partial]
          }
        }
      }
    }
  }

  *compoundsByRules(
    wordRest: string,
    prev: Word[] = [],
    rules: null | Set<CompoundRule> = null,
    allowNoSuggest = true
  ): Generator<CompoundForm> {
    const aff = this.aff

    if (rules === null) {
      rules = this.aff.COMPOUNDRULE
    }

    if (prev.length) {
      for (const homonym of this.dic.homonyms(wordRest)) {
        const parts = [...prev, homonym]
        const flagSets = iterate(parts)
          .filter(word => Boolean(word.flags))
          .map(word => word.flags!)
          .toSet()
        for (const rule of rules) {
          if (rule.match(flagSets)) {
            yield [new AffixForm(wordRest, wordRest)]
          }
        }
      }
    }

    if (
      wordRest.length < aff.COMPOUNDMIN &&
      aff.COMPOUNDWORDMAX &&
      prev.length >= aff.COMPOUNDWORDMAX
    ) {
      return
    }

    for (let pos = aff.COMPOUNDMIN; pos < wordRest.length - aff.COMPOUNDMIN + 1; pos++) {
      const beg = wordRest.slice(0, pos)

      for (const homonynm of this.dic.homonyms(beg)) {
        const parts = [...prev, homonynm]
        const flagSets = iterate(parts)
          .filter(word => Boolean(word.flags))
          .map(word => word.flags!)
          .toSet()
        const compoundRules = iterate(rules)
          .filter(rule => rule.match(flagSets, true))
          .toSet()
        if (compoundRules.size) {
          for (const rest of this.compoundsByRules(
            wordRest.slice(pos),
            parts,
            compoundRules,
            allowNoSuggest
          )) {
            yield [new AffixForm(beg, beg), ...rest]
          }
        }
      }
    }
  }

  isBadCompound(compound: CompoundForm, captype: CapType) {
    const aff = this.aff

    if (aff.FORCEUCASE && captype !== CapType.ALL && captype !== CapType.INIT) {
      if (this.dic.hasFlag(compound[compound.length - 1].text, aff.FORCEUCASE)) {
        return true
      }
    }

    return [...compound]
      .reverse()
      .slice(-1)
      .some((leftParadigm, idx) => {
        const left = leftParadigm.text
        const rightParadigm = compound[idx + 1]
        const right = rightParadigm.text

        if (aff.COMPOUNDFORBIDFLAG) {
          if (this.dic.hasFlag(left, aff.COMPOUNDFORBIDFLAG)) {
            return true
          }
        }

        if (any(this.affixForms(`${left} ${right}`, captype))) return true

        if (aff.CHECKCOMPOUNDREP) {
          for (const candidate of replchars(left + right, aff.REP)) {
            if (
              typeof candidate === "string" &&
              any(this.affixForms(candidate, captype))
            ) {
              return true
            }
          }
        }

        if (aff.CHECKCOMPOUNDTRIPLE) {
          if (
            `${left.slice(-2)}${right.slice(0, 1)}`.length === 1 ||
            `${left.slice(-1)}${right.slice(2)}`.length === 1
          ) {
            return true
          }
        }

        if (aff.CHECKCOMPOUNDCASE) {
          const rightC = right[0]
          const leftC = left[left.length - 1]
          if (
            (isUppercased(rightC) || isUppercased(leftC)) &&
            rightC !== "-" &&
            leftC !== "-"
          ) {
            return true
          }

          if (aff.CHECKCOMPOUNDPATTERN) {
            for (const pattern of aff.CHECKCOMPOUNDPATTERN) {
              if (pattern.match(leftParadigm, rightParadigm)) {
                return true
              }
            }
          }

          if (aff.CHECKCOMPOUNDUP) {
            if (left === right && idx === compound.length - 2) {
              return true
            }
          }
        }
      })
  }
}
