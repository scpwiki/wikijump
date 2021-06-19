import iterate from "iterare"
import type { Aff, Flags } from "../aff"
import { CapType } from "../aff/casing"
import type { CompoundRule } from "../aff/compound-rule"
import type { Dic } from "../dic"
import type { Word } from "../dic/word"
import { replchars } from "../permutations"
import { any, includes, isUppercased, lowercase } from "../util"
import { deprefix, desuffix } from "./decompose"
import { AffixForm, CompoundForm, CompoundPos } from "./forms"
import { LKWord } from "./lk-word"

const NUMBER_REGEX = /^\d+(\.\d+)?$/

export interface LookupResult {
  correct: boolean
  forbidden: boolean
  warn: boolean
}

export interface LKC {
  caps?: boolean
  allowNoSuggest?: boolean
  affixForms?: boolean
  compoundForms?: boolean
  withForbidden?: boolean
}

export interface LKFlags {
  prefix: Flags
  suffix: Flags
  forbidden: Flags
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
      if (!this.correct(word2, { caps, allowNoSuggest })) {
        return { correct: false, forbidden, warn }
      }
    }

    return { correct: true, forbidden, warn }
  }

  /**
   * Checks if a word is spelled correctly. Performs no processing on the
   * word, such as handling `aff.IGNORE` characters.
   *
   * @param word - The word to check.
   * @param caps - If true, checking will be case sensitive. Defaults to true.
   * @param allowNoSuggest - If false, words which are in the dictionary,
   *   but are flagged with the `NOSUGGEST` flag (if provided), will not be
   *   considered correct. Defaults to true.
   * @param compounds - If provided, this boolean will toggle between
   *   checking only affix forms or compound forms.
   */
  correct(word: string, { caps, allowNoSuggest, affixForms, compoundForms }: LKC = {}) {
    return any(this.goodForms(word, { caps, allowNoSuggest, affixForms, compoundForms }))
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
  *breakWord(text: string, depth = 0): Iterable<string[]> {
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
    {
      caps = true,
      allowNoSuggest = true,
      affixForms = true,
      compoundForms = true
    }: LKC = {}
  ) {
    let captype: CapType, variants: string[]
    if (caps) {
      ;[captype, ...variants] = this.aff.casing.variants(word)
    } else {
      captype = this.aff.casing.guess(word)
      variants = [word]
    }

    const lkword = new LKWord(word, captype)

    for (const variant of variants) {
      if (affixForms) {
        for (const form of this.affixForms(lkword.to(variant), { allowNoSuggest })) {
          if (
            form.inDictionary &&
            captype === CapType.ALL &&
            includes(this.aff.KEEPCASE, form.flags) &&
            this.aff.isSharps(form.inDictionary.stem) &&
            this.aff.isSharps(lkword.word)
          ) {
            continue
          }
          yield form
        }
      }
      if (compoundForms) {
        yield* this.compoundForms(lkword, { allowNoSuggest })
      }
    }
  }

  // -- AFFIX FORMS

  *affixForms(
    lkword: LKWord,
    { allowNoSuggest = true, withForbidden = false }: LKC = {},
    flags: LKFlags = { prefix: new Set(), suffix: new Set(), forbidden: new Set() }
  ) {
    const candidates = (form: AffixForm, stem: string, caps = false, words?: Set<Word>) =>
      this.candidates(form, lkword, { allowNoSuggest, caps }, stem, words)

    for (const form of this.produceAffixForms(lkword, flags)) {
      let found = false

      const homonyms = this.dic.homonyms(form.stem)

      if (homonyms.size) {
        if (
          !withForbidden &&
          this.aff.FORBIDDENWORD &&
          (lkword.pos !== undefined || form.hasAffixes) &&
          iterate(homonyms).some(({ flags }) => includes(this.aff.FORBIDDENWORD, flags))
        ) {
          return
        }

        for (const candidate of candidates(form, form.stem, false, homonyms)) {
          yield candidate
        }
      }

      if (
        lkword.pos === CompoundPos.BEGIN &&
        this.aff.FORCEUCASE &&
        lkword.type === CapType.INIT
      ) {
        for (const candidate of candidates(form, lowercase(form.stem))) {
          yield candidate
        }
      }

      if (found || lkword.pos !== undefined || lkword.type !== CapType.ALL) {
        continue
      }

      if (this.aff.casing.guess(lkword.word) === CapType.NO) {
        for (const candidate of candidates(form, form.stem, true)) {
          yield candidate
        }
      }
    }
  }

  *candidates(
    form: AffixForm,
    lkword: LKWord,
    { allowNoSuggest = true, caps = true }: LKC = {},
    stem = form.stem,
    homonyms?: Set<Word>
  ) {
    const aff = this.aff
    for (const homonym of homonyms ?? this.dic.homonyms(stem, !caps)) {
      const candidate = form.replace({ inDictionary: homonym })

      if (!candidate.inDictionary) continue

      const rootFlags = candidate.inDictionary.flags ?? new Set()
      const allFlags = candidate.flags

      if (!allowNoSuggest && includes(aff.NOSUGGEST, rootFlags)) continue

      if (
        lkword.type !== candidate.inDictionary.capType &&
        includes(aff.KEEPCASE, rootFlags) &&
        !aff.isSharps(candidate.inDictionary.stem)
      ) {
        continue
      }

      if (aff.NEEDAFFIX) {
        if (candidate.hasAffixes) {
          if (candidate.affixes().every(affix => affix.flags.has(aff.NEEDAFFIX!))) {
            continue
          }
        } else if (rootFlags.has(aff.NEEDAFFIX)) {
          continue
        }
      }

      if (candidate.prefix && !allFlags.has(candidate.prefix.flag)) continue
      if (candidate.suffix && !allFlags.has(candidate.suffix.flag)) continue

      if (aff.CIRCUMFIX) {
        const suffixHas = Boolean(candidate.suffix?.flags.has(aff.CIRCUMFIX))
        const prefixHas = Boolean(candidate.prefix?.flags.has(aff.CIRCUMFIX))
        if (suffixHas !== prefixHas) continue
      }

      if (lkword.pos === undefined) {
        if (!includes(aff.ONLYINCOMPOUND, allFlags)) yield candidate
        continue
      }

      if (includes(aff.COMPOUNDFLAG, allFlags)) {
        yield candidate
        continue
      }

      let passes = false
      // prettier-ignore
      switch(lkword.pos) {
        case CompoundPos.BEGIN:  passes = includes(aff.COMPOUNDBEGIN,  allFlags)
        case CompoundPos.MIDDLE: passes = includes(aff.COMPOUNDMIDDLE, allFlags)
        case CompoundPos.END:    passes = includes(aff.COMPOUNDEND,    allFlags)
      }

      if (passes) yield candidate
    }
  }

  *produceAffixForms(lkword: LKWord, flags: LKFlags) {
    yield new AffixForm(lkword.word)

    const suffixAllowed =
      lkword.pos === undefined || lkword.pos === CompoundPos.END || flags.suffix.size

    const prefixAllowed =
      lkword.pos === undefined || lkword.pos === CompoundPos.BEGIN || flags.prefix.size

    if (suffixAllowed) {
      yield* desuffix(this.aff, lkword.word, flags.suffix, flags.forbidden)
    }

    if (prefixAllowed) {
      for (const form of deprefix(this.aff, lkword.word, flags.prefix, flags.forbidden)) {
        yield form
        if (suffixAllowed && form.prefix?.crossproduct) {
          for (const form2 of desuffix(
            this.aff,
            form.stem,
            flags.suffix,
            flags.forbidden,
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

  *compoundForms(lkword: LKWord, { allowNoSuggest = true }: LKC = {}) {
    if (this.aff.FORBIDDENWORD) {
      for (const candidate of this.affixForms(lkword, { withForbidden: true })) {
        if (candidate.flags.has(this.aff.FORBIDDENWORD)) return
      }
    }

    if (this.aff.COMPOUNDBEGIN || this.aff.COMPOUNDFLAG) {
      for (const compound of this.compoundsByFlags(lkword, allowNoSuggest)) {
        if (!this.isBadCompound(compound, lkword.type)) {
          yield compound
        }
      }
    }

    if (this.aff.COMPOUNDRULE) {
      for (const compound of this.compoundsByRules(lkword, allowNoSuggest)) {
        if (!this.isBadCompound(compound, lkword.type)) {
          yield compound
        }
      }
    }
  }

  *compoundsByFlags(
    lkword: LKWord,
    allowNoSuggest = true,
    depth = 0
  ): Iterable<CompoundForm> {
    const aff = this.aff

    const forbiddenFlags = new Set<string>()
    const permitFlags = new Set<string>()

    if (aff.COMPOUNDFORBIDFLAG) forbiddenFlags.add(aff.COMPOUNDFORBIDFLAG)
    if (aff.COMPOUNDPERMITFLAG) permitFlags.add(aff.COMPOUNDPERMITFLAG)

    if (depth) {
      for (const form of this.affixForms(
        lkword,
        { allowNoSuggest },
        { prefix: permitFlags, suffix: new Set(), forbidden: forbiddenFlags }
      )) {
        yield [form]
      }
    }

    if (lkword.length < aff.COMPOUNDMIN * 2) return
    if (aff.COMPOUNDWORDMAX && depth > aff.COMPOUNDWORDMAX) return

    const compoundpos = depth ? CompoundPos.MIDDLE : CompoundPos.BEGIN
    const prefixFlags =
      compoundpos === CompoundPos.BEGIN ? new Set<string>() : permitFlags

    for (let pos = aff.COMPOUNDMIN; pos < lkword.length - aff.COMPOUNDMIN + 1; pos++) {
      const beg = lkword.slice(0, pos)
      beg.pos = compoundpos

      const rest = lkword.slice(pos)
      rest.pos = compoundpos

      for (const form of this.affixForms(
        beg,
        { allowNoSuggest },
        { prefix: prefixFlags, suffix: permitFlags, forbidden: forbiddenFlags }
      )) {
        for (const partial of this.compoundsByFlags(rest, allowNoSuggest, depth + 1)) {
          yield [form, ...partial]
        }
      }

      if (aff.SIMPLIFIEDTRIPLE && beg.at(-1) === rest.at(0)) {
        for (const form of this.affixForms(
          beg.add(beg.at(-1)),
          { allowNoSuggest },
          { prefix: prefixFlags, suffix: permitFlags, forbidden: forbiddenFlags }
        )) {
          for (const partial of this.compoundsByFlags(rest, allowNoSuggest, depth + 1)) {
            yield [form.replace({ text: beg.word }), ...partial]
          }
        }
      }
    }
  }

  *compoundsByRules(
    lkword: LKWord,
    allowNoSuggest = true,
    prev: Word[] = [],
    rules: null | Set<CompoundRule> = null
  ): Iterable<CompoundForm> {
    const aff = this.aff

    if (rules === null) rules = this.aff.COMPOUNDRULE

    if (prev.length) {
      for (const homonym of this.dic.homonyms(lkword.word)) {
        const parts = [...prev, homonym]
        const flagSets = iterate(parts)
          .filter(word => Boolean(word.flags))
          .map(word => word.flags!)
          .toSet()
        for (const rule of rules) {
          if (rule.match(flagSets)) {
            yield [new AffixForm(lkword.word)]
          }
        }
      }
    }

    if (lkword.length < aff.COMPOUNDMIN * 2) return
    if (aff.COMPOUNDWORDMAX && prev.length >= aff.COMPOUNDWORDMAX) return

    for (let pos = aff.COMPOUNDMIN; pos < lkword.length - aff.COMPOUNDMIN + 1; pos++) {
      const beg = lkword.slice(0, pos)

      for (const homonynm of this.dic.homonyms(beg.word)) {
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
            lkword.slice(pos),
            allowNoSuggest,
            parts,
            compoundRules
          )) {
            yield [new AffixForm(beg.word), ...rest]
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

        if (this.dic.hasFlag(left, aff.COMPOUNDFORBIDFLAG)) {
          return true
        }

        if (any(this.affixForms(new LKWord(`${left} ${right}`, captype)))) return true

        if (aff.CHECKCOMPOUNDREP) {
          for (const candidate of replchars(left + right, aff.REP)) {
            if (typeof candidate !== "string") continue
            if (any(this.affixForms(new LKWord(candidate, captype)))) {
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
