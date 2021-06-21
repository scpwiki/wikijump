import iterate from "iterare"
import type { LKFlags } from "."
import type { Aff } from "../aff"
import { CompoundPos } from "../constants"
import { concat, reverse } from "../util"
import { AffixForm } from "./forms"
import type { LKWord } from "./lk-word"

export const enum AffixType {
  PREFIX,
  SUFFIX
}

/**
 * Yields permutations of a word split up (with whitespace) using the
 * `BREAK` rules given by the spellchecker's {@link Aff} data.
 *
 * @param aff - The affix data to use.
 * @param text - The word/text to split.
 * @param depth - The current depth of the check. Used by this function
 *   when calling itself recursively. There isn't any need to set it yourself.
 */
export function* breakWord(aff: Aff, text: string, depth = 0): Iterable<string> {
  if (depth > 10) return
  yield text
  for (const pattern of aff.BREAK) {
    for (const m of text.matchAll(pattern)) {
      const start = text.slice(0, m.index!)
      const rest = text.slice(0, m.index! + m[0].length)
      for (const breaking of breakWord(aff, rest, depth + 1)) {
        yield start
        yield* breaking
      }
    }
  }
}

/**
 * Takes in a {@link LKWord} and yields a progressive decomposition of the
 * affixes and stems that can be found in the word.
 *
 * @param lkword - The word to decompose.
 * @param flags - The {@link LKFlags} that restrain the possible forms of the word.
 */
export function* decompose(aff: Aff, lkword: LKWord, flags: LKFlags) {
  yield new AffixForm(lkword)

  const suffixAllowed =
    lkword.pos === undefined || lkword.pos === CompoundPos.END || flags.suffix.size

  const prefixAllowed =
    lkword.pos === undefined || lkword.pos === CompoundPos.BEGIN || flags.prefix.size

  if (suffixAllowed) {
    yield* desuffix(aff, lkword.word, flags)
  }

  if (prefixAllowed) {
    for (const form of deprefix(aff, lkword.word, flags)) {
      yield form
      if (suffixAllowed && form.prefix?.crossproduct) {
        for (const form2 of desuffix(aff, form.stem, flags, true)) {
          yield form2.replace({ text: form.text, prefix: form.prefix })
        }
      }
    }
  }
}

/**
 * Gets the affixes for a word, either yielding {@link Prefix}es or
 * {@link Suffix}es depending on the given {@link AffixType}.
 *
 * @param aff - The {@link Aff} data to use, specifically the trie indexes.
 * @param type - The {@link AffixType} being searched for, either
 *   {@link AffixType.PREFIX} or {@link AffixType.SUFFIX}.
 * @param word - The word to get the affixes of.
 * @param flags - The flags used for filtering what is yielded.
 * @param crossproduct - If true, enables crossproduct checking.
 */
export function* affixes(
  aff: Aff,
  type: AffixType,
  word: string,
  flags: LKFlags,
  crossproduct?: boolean
) {
  const isSuffix = type === AffixType.SUFFIX
  const segments = isSuffix
    ? aff.suffixesIndex.segments(reverse(word))
    : aff.prefixesIndex.segments(word)

  if (segments) {
    const required = isSuffix ? flags.suffix : flags.prefix

    yield* iterate(segments)
      .flatten()
      .filter(affix => {
        if (isSuffix && !(crossproduct || affix.crossproduct)) return false
        if (!affix.compatible(required, flags.forbidden)) return false
        return affix.on(word)
      })
  }
}

/**
 * Yields progressively more decomposed transformations (more suffixes
 * removed) of the given word as {@link AffixForm}s.
 *
 * @param aff - The {@link Aff} data to use.
 * @param word - The word to decompose the suffixes out of.
 * @param flags - The flags used to filter valid {@link AffixForm}s.
 * @param crossproduct - If true, crossproduct checking will be enabled.
 * @param nested - Internal argument for handling recursion.
 */
export function* desuffix(
  aff: Aff,
  word: string,
  flags: LKFlags,
  crossproduct?: boolean,
  nested?: boolean
): Iterable<AffixForm> {
  for (const suffix of affixes(aff, AffixType.SUFFIX, word, flags, crossproduct)) {
    const stem = suffix.apply(word)

    yield new AffixForm(word, stem, { suffix })

    if (!nested) {
      const newFlags = { ...flags, suffix: concat(suffix.flags, flags.suffix) }
      for (const form2 of desuffix(aff, stem, newFlags, crossproduct, true)) {
        yield form2.replace({ text: word, suffix2: suffix })
      }
    }
  }
}

/**
 * Yields progressively more decomposed transformations (more prefixes
 * removed) of the given word as {@link AffixForm}s.
 *
 * @param aff - The {@link Aff} data to use.
 * @param word - The word to decompose the prefixes out of.
 * @param flags - The flags used to filter valid {@link AffixForm}s.
 * @param nested - Internal argument for handling recursion.
 */
export function* deprefix(
  aff: Aff,
  word: string,
  flags: LKFlags,
  nested?: boolean
): Iterable<AffixForm> {
  for (const prefix of affixes(aff, AffixType.PREFIX, word, flags)) {
    const stem = prefix.apply(word)

    yield new AffixForm(word, stem, { prefix })

    if (!nested && aff.COMPLEXPREFIXES) {
      const newFlags = { ...flags, prefix: concat(prefix.flags, flags.prefix) }
      for (const form2 of deprefix(aff, stem, newFlags, true)) {
        yield form2.replace({ text: word, prefix2: prefix })
      }
    }
  }
}
