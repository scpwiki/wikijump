import iterate from "iterare"
import type { PrefixMap, SuffixMap } from "../aff"
import type { Prefix, Suffix } from "../aff/affix"
import { CONSTANTS as C } from "../constants"
import type { Word } from "../dic/word"
import { lowercase } from "../util"
import {
  preciseAffixScore,
  rootScore,
  roughAffixScore,
  ScoresList,
  scoreThreshold
} from "./scores"

export class NgramSuggestionBuilder {
  /** The "root" scores for potential suggestions for a misspelling. */
  private declare roots: ScoresList<[Word]>

  constructor(
    /** The misspelling that suggestions are being built for. */
    private misspelling: string,
    /** The {@link PrefixMap} to use when assembling suggestions. */
    private prefixes: PrefixMap,
    /** The {@link SuffixMap} to use when assembling suggestions. */
    private suffixes: SuffixMap,
    /** A set of already known suggestions that should be skipped. */
    private known: Set<string>,
    /**
     * Sets the similarity factor.
     *
     * - `5`: The default value.
     * - `0`: Fewer ngram suggestions, but always at least one.
     * - `10`: Maximum value, yields `MAXNGRAMSUGS` number of suggestions.
     */
    private maxDiff: number,
    /**
     * If true, all bad ngram suggestions will be removed, rather than
     * keeping at least one.
     */
    private onlyMaxDiff = false,
    /**
     * Produces less suggestions if true, so that phonetic suggestion
     * system isn't skipped due to a large amount of ngram suggestions.
     */
    private hasPhonetic = false
  ) {
    this.misspelling = misspelling
    this.prefixes = prefixes
    this.suffixes = suffixes
    this.known = known
    this.maxDiff = maxDiff
    this.onlyMaxDiff = onlyMaxDiff
    this.hasPhonetic = hasPhonetic
    this.roots = new ScoresList<[Word]>(C.NGRAM_MAX_ROOTS)
  }

  /**
   * Steps the builder forward by providing another {@link Word} to process.
   *
   * @param word - The {@link Word} to process.
   */
  step(word: Word) {
    if (Math.abs(word.stem.length - this.misspelling.length) > 4) return

    let score = rootScore(this.misspelling, word.stem)

    if (word.altSpellings?.size) {
      for (const variant of word.altSpellings) {
        score = Math.max(score, rootScore(this.misspelling, variant))
      }
    }

    this.roots.add(score, word)
  }

  /** Finishes the builder and yields the resulting suggestions (as strings). */
  *finish() {
    const threshold = scoreThreshold(this.misspelling)

    const guesses = new ScoresList<[string, string]>(C.NGRAM_MAX_GUESSES)

    for (const [root] of this.roots.finish()) {
      if (root.altSpellings?.size) {
        for (const variant of root.altSpellings) {
          const lower = lowercase(variant)
          const score = roughAffixScore(this.misspelling, variant)
          if (score > threshold) guesses.add(score, lower, root.stem)
        }
      }

      for (const form of this.formsFor(root, this.misspelling)) {
        const lower = lowercase(form)
        const score = roughAffixScore(this.misspelling, form)
        if (score > threshold) guesses.add(score, lower, form)
      }
    }

    const fact = this.maxDiff >= 0 ? (10 - this.maxDiff) / 5 : 1

    yield* this.filterGuesses(
      guesses.finish(
        ([score, compared, real]) =>
          [
            preciseAffixScore(this.misspelling, compared, fact, score, this.hasPhonetic),
            real
          ] as [number, string],
        true
      )
    )
  }

  /**
   * Returns the forms (permutations) of a {@link Word}, with all valid
   * suffixes and prefixes.
   *
   * @param word - The {@link Word} to get the forms of.
   * @param similarTo - The string/word that the forms found should be similar to.
   */
  private formsFor(word: Word, similarTo: string) {
    const res: string[] = [word.stem]

    const suffixes = !word.flags
      ? []
      : iterate(word.flags)
          .filter(flag => this.suffixes.has(flag))
          .map(flag => this.suffixes.get(flag)!)
          .flatten()
          .filter(suffix => suffix.relevant(word.stem) && similarTo.endsWith(suffix.add))
          .toArray()

    const prefixes = !word.flags
      ? []
      : iterate(word.flags)
          .filter(flag => this.prefixes.has(flag))
          .map(flag => this.prefixes.get(flag)!)
          .flatten()
          .filter(
            prefix => prefix.relevant(word.stem) && similarTo.startsWith(prefix.add)
          )
          .toArray()

    const cross = iterate(prefixes)
      .map(prefix =>
        iterate(suffixes)
          .filter(suffix => suffix.crossproduct && prefix.crossproduct)
          .map(suffix => [prefix, suffix] as [Prefix, Suffix])
          .toArray()
      )
      .flatten()
      .toArray()

    for (const suffix of suffixes) {
      const root = suffix.strip ? word.stem.slice(0, -suffix.strip.length) : word.stem
      res.push(root + suffix.add)
    }

    for (const [prefix, suffix] of cross) {
      const root = suffix.strip
        ? word.stem.slice(prefix.strip.length, -suffix.strip.length)
        : word.stem.slice(prefix.strip.length)
      res.push(prefix.add + root + suffix.add)
    }

    for (const prefix of prefixes) {
      const root = word.stem.slice(prefix.strip.length)
      res.push(prefix.add + root)
    }

    return res
  }

  /**
   * Filters out terrible guesses based on their score or if they were already known.
   *
   * @param guesses - A list of tuples, containing a score and guess, in that order.
   */
  private *filterGuesses(guesses: [number, string][]) {
    let seen = false
    let found = 0

    for (const [score, value] of guesses) {
      if (seen && score <= 1000) return

      if (score > 1000) seen = true
      else if (score < -100) {
        if (found > 0 || this.onlyMaxDiff) return
        seen = true
      }

      if (!iterate(this.known).some(word => word.includes(value))) {
        found++
        yield value
      }
    }
  }
}
