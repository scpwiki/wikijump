import iterate from "iterare"
import type { Aff, Flags } from "../aff"
import type { CompoundRule } from "../aff/compound-rule"
import { CapType, CompoundPos, CONSTANTS as C } from "../constants"
import type { Dic } from "../dic"
import type { Word } from "../dic/word"
import { replchars } from "../permutations"
import { any, includes, isUppercased, lowercase } from "../util"
import { breakWord, decompose } from "./decompose"
import { AffixForm, CompoundForm } from "./forms"
import { LKWord } from "./lk-word"

/** The resulting data returned from executing a lookup. */
export interface LookupResult {
  /** Indicates if the word was spelled correctly. */
  correct: boolean
  /**
   * Indicates if the word was marked as forbidden with the spellchecker's
   * dictionary.
   */
  forbidden: boolean
  /**
   * Indicates if the word was marked as "warn" in the dictionary, which
   * probably means that the word is *technically* valid but is still
   * likely to have been a mistake.
   */
  warn: boolean
}

/**
 * {@link Lookup} context. Many methods in the {@link Lookup} class share
 * these options.
 */
export interface LKC {
  /** If true, lookups will be case sensitive. */
  caps?: boolean

  /**
   * If false, words which are in the dictionary, but are flagged with the
   * `NOSUGGEST` flag (if provided), will not be considered correct.
   * Defaults to true.
   */
  allowNoSuggest?: boolean

  /**
   * Used by {@link Lookup.forms}. If false, {@link AffixForm} instances
   * won't be yielded. Defaults to true.
   */
  affixForms?: boolean

  /**
   * Used by {@link Lookup.forms}. If false, {@link CompoundForm} instances
   * won't be yielded. Defaults to true.
   */
  compoundForms?: boolean

  /**
   * Used by {@link Lookup.affixForms}. If true, {@link AffixForm}s that have
   * the `FORBIDDENWORD` flag will still be yielded.
   */
  withForbidden?: boolean
}

/** Context object for handling {@link AffixForm} decomposing state. */
export interface LKFlags {
  /** The set of prefix flags currently in the state. */
  prefix: Flags

  /** The set of suffix flags currently in the state. */
  suffix: Flags

  /** A set of flags that invalidates {@link AffixForm}s if they have one of them. */
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
  check(word: string, caps = true, allowNoSuggest = true): LookupResult {
    let forbidden = this.isForbidden(word)
    let warn = this.isWarn(word)

    if (forbidden) return { correct: false, forbidden, warn }

    if (this.aff.ICONV) word = this.aff.ICONV.match(word)

    if (this.aff.IGNORE) {
      for (const ch of this.aff.IGNORE) {
        word = word.replaceAll(ch, "")
      }
    }

    if (C.NUMBER_REGEX.test(word)) return { correct: true, forbidden, warn }

    for (const word2 of breakWord(this.aff, word)) {
      if (!this.correct(word2, { caps, allowNoSuggest })) {
        return { correct: false, forbidden, warn }
      }
    }

    return { correct: true, forbidden, warn }
  }

  /**
   * Yields *correct* combinations of stems and affixes for a word,
   * specifically instances of {@link AffixForm} or {@link CompoundForm}. If
   * this function does actually yield a form, that means that it can be
   * considered as spelled correctly.
   *
   * @param word - The word to yield the forms of.
   * @see {@link LKC}
   */
  *forms(
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

  /**
   * Yields the allowed {@link AffixForm}s for a word, as in all ways the
   * word can be split into stems and affixes, with all stems and affixes
   * being mutually compatible.
   *
   * @param lkword - The {@link LKWord} to yield the forms of.
   * @see {@link LKC}
   * @see {@link LKFlags}
   */
  *affixForms(
    lkword: LKWord,
    { allowNoSuggest = true, withForbidden = false }: LKC = {},
    flags: LKFlags = { prefix: new Set(), suffix: new Set(), forbidden: new Set() }
  ) {
    const candidates = (form: AffixForm, stem: string, caps = false, words?: Set<Word>) =>
      this.candidates(form, lkword, { allowNoSuggest, caps }, stem, words)

    for (const form of decompose(this.aff, lkword, flags)) {
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

  /**
   * Takes an {@link AffixForm} and an {@link LKWord} and yields all of the
   * allowed forms of the entire word form, taking into account {@link Aff}
   * directives and other edge cases.
   *
   * @param form - The base {@link AffixForm} to start with.
   * @param lkword - The base {@link LKWord} to start with.
   * @param stem - The word stem to use. If this isn't provided, it will
   *   default to the {@link AffixForm.stem}.
   * @param homonyms - If for some reason you have already searched the
   *   dictionary for the given stem's homonyms, you can supply that result
   *   here to prevent another dictionary search.
   * @see {@link LKC}
   */
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
          if (candidate.affixes().every(affix => affix.has(aff.NEEDAFFIX))) {
            continue
          }
        } else if (rootFlags.has(aff.NEEDAFFIX)) {
          continue
        }
      }

      if (candidate.prefix && !allFlags.has(candidate.prefix.flag)) continue
      if (candidate.suffix && !allFlags.has(candidate.suffix.flag)) continue

      if (aff.CIRCUMFIX) {
        const suffixHas = Boolean(candidate.suffix?.has(aff.CIRCUMFIX))
        const prefixHas = Boolean(candidate.prefix?.has(aff.CIRCUMFIX))
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

  // -- COMPOUND FORMS

  /**
   * Produces all valid {@link CompoundForm}s for a word. Really, the "hard
   * work" done by this function is performed by the
   * {@link Lookup.compoundsByFlags} and the {@link Lookup.compoundsByRules} methods.
   *
   * @param lkword - The {@link LKWord} to get the {@link CompoundForm}s from.
   * @see {@link LKC}
   */
  *compoundForms(lkword: LKWord, { allowNoSuggest = true }: LKC = {}) {
    // don't even try to decompose a forbidden word
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

  /**
   * Takes a word and yields the {@link CompoundForm}s of it using the
   * `COMPOUNDFLAG`/`COMPOUNDBEGIN|MIDDLE|END` marker system.
   *
   * @param lkword - The word to yield the {@link CompoundForm}s of.
   * @param allowNoSuggest - See {@link LKC}.
   * @param depth - Internal argument for doing recursion. Prevents absurd
   *   generation of compound forms.
   * @see {@link LKC}
   */
  *compoundsByFlags(
    lkword: LKWord,
    allowNoSuggest = true,
    depth = 0
  ): Iterable<CompoundForm> {
    const aff = this.aff

    const forbiddenFlags: Flags = new Set<string>()
    const permitFlags: Flags = new Set<string>()

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

  /**
   * Takes a word and yields the {@link CompoundForm}s of it using the
   * `COMPOUNDRULE` pattern system.
   *
   * @param lkword - The word to yield the {@link CompoundForm}s of.
   * @param allowNoSuggest - See {@link LKC}.
   * @param prev - Internal argument for specifying the previous parts of
   *   the compound when doing recursion.
   * @param rules - Internal argument for specifying the current
   *   `COMPOUNDRULE` set when doing recursion.
   * @see {@link LKC}
   * @see {@link CompoundRule}
   */
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
            yield [new AffixForm(lkword)]
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
            yield [new AffixForm(beg), ...rest]
          }
        }
      }
    }
  }

  // -- UTILITY

  /**
   * Checks if a word is spelled correctly. Performs no processing on the
   * word, such as handling `aff.IGNORE` characters.
   *
   * @param word - The word to check.
   * @see {@link LKC}
   */
  correct(word: string, { caps, allowNoSuggest, affixForms, compoundForms }: LKC = {}) {
    return any(this.forms(word, { caps, allowNoSuggest, affixForms, compoundForms }))
  }

  /**
   * Determines if a word is marked with the `WARN` flag.
   *
   * @param word - The word to check.
   */
  isWarn(word: string) {
    return this.dic.hasFlag(word, this.aff.WARN, true)
  }

  /**
   * Determines if a word is marked as forbidden, either through the
   * `FORBIDDENWORD` flag *or* the the combination of the word having the
   * `WARN` flag and the `FORBIDWARN` directive being true.
   *
   * @param word - The word to check.
   */
  isForbidden(word: string) {
    return (
      this.dic.hasFlag(word, this.aff.FORBIDDENWORD, true) ||
      (this.aff.FORBIDWARN && this.dic.hasFlag(word, this.aff.WARN, true))
    )
  }

  /**
   * Determines if a {@link CompoundForm} is invalid, by various criteria.
   *
   * @param compound - The {@link CompoundForm} to check.
   * @param captype - The {@link CapType} of the original word.
   * @see {@link CompoundPattern}
   */
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
