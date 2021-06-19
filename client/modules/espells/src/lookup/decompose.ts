import iterate from "iterare"
import type { Aff, Flags } from "../aff"
import { Prefix, Suffix } from "../aff/affix"
import { reverse } from "../util"
import { AffixForm } from "./forms"

export function isGoodAffix(
  affix: Prefix | Suffix,
  word: string,
  requiredFlags: Flags,
  forbiddenFlags: Flags,
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

export function* desuffix(
  aff: Aff,
  word: string,
  requiredFlags: Flags,
  forbiddenFlags: Flags,
  nested = false,
  crossproduct = false
): Iterable<AffixForm> {
  const segments = aff.suffixesIndex.segments(reverse(word))

  if (segments) {
    const possibleSuffixes = iterate(segments)
      .flatten()
      .filter(suffix =>
        isGoodAffix(suffix, word, requiredFlags, forbiddenFlags, crossproduct)
      )

    for (const suffix of possibleSuffixes) {
      const stem = word.replace(suffix.replaceRegex, suffix.strip)
      yield new AffixForm(word, stem, { suffix })
      if (!nested) {
        for (const form2 of desuffix(
          aff,
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

export function* deprefix(
  aff: Aff,
  word: string,
  requiredFlags: Flags,
  forbiddenFlags: Flags,
  nested = false
): Iterable<AffixForm> {
  const segments = aff.prefixesIndex.segments(word)

  if (segments) {
    const possiblePrefixes = iterate(segments)
      .flatten()
      .filter(prefix => isGoodAffix(prefix, word, requiredFlags, forbiddenFlags))

    for (const prefix of possiblePrefixes) {
      const stem = word.replace(prefix.replaceRegex, prefix.strip)
      yield new AffixForm(word, stem, { prefix })
      if (!nested && aff.COMPLEXPREFIXES) {
        for (const form2 of deprefix(
          aff,
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
