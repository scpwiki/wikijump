import iterate from "iterare"
import type { Aff } from "../aff"
import { CapType, CONSTANTS as C } from "../constants"
import type { Dic } from "../dic"
import type { Word } from "../dic/word"
import type { Lookup } from "../lookup"
import {
  badchar,
  badcharkey,
  doubletwochars,
  longswapchar,
  mapchars,
  replchars,
  swapchar,
  twowords
} from "../permutations"
import { intersect, lowercase, uppercase } from "../util"
import { ngramSuggest } from "./ngram"
import { phonetSuggest } from "./phonet"
import { MultiWordSuggestion, Suggestion } from "./suggestion"

type Handler = (suggestion: Suggestion, checkInclusion?: boolean) => Iterable<Suggestion>

export class Suggest {
  private declare aff: Aff
  private declare dic: Dic
  private declare lookup: Lookup
  private declare ngramWords: Set<Word>
  private declare dashes: boolean

  constructor(aff: Aff, dic: Dic, lookup: Lookup) {
    this.aff = aff
    this.dic = dic
    this.lookup = lookup

    const badFlags = iterate([aff.FORBIDDENWORD, aff.NOSUGGEST, aff.ONLYINCOMPOUND])
      .filter(flag => Boolean(flag))
      .toSet()

    this.ngramWords = iterate(dic.words)
      .filter(word => (!word.flags ? true : intersect(word.flags, badFlags).size === 0))
      .toSet()

    this.dashes = aff.TRY.includes("-") || aff.TRY.includes("a")
  }

  private correct(word: string, compounds?: boolean) {
    return this.lookup.correct(word, {
      caps: false,
      allowNoSuggest: false,
      affixForms: !compounds,
      compoundForms: compounds
    })
  }

  private isForbidden(word: string) {
    return this.dic.hasFlag(word, this.aff.FORBIDDENWORD)
  }

  private *handle(
    word: string,
    captype: CapType,
    handled: Set<string>,
    suggestion: Suggestion,
    checkInclusion = false
  ) {
    let text = suggestion.text

    if (!this.dic.hasFlag(text, this.aff.KEEPCASE) || this.aff.isSharps(text)) {
      text = this.aff.casing.coerce(text, captype)
      // revert if forbidden
      if (text !== suggestion.text && this.isForbidden(text)) {
        text = suggestion.text
      }

      if (captype === CapType.HUH || captype === CapType.HUHINIT) {
        const pos = text.indexOf(" ")
        if (pos !== -1) {
          if (text[pos + 1] !== word[pos] && uppercase(text[pos + 1]) === word[pos]) {
            text = text.slice(0, pos + 1) + word[pos] + word.slice(pos + 2)
          }
        }
      }
    }

    if (this.isForbidden(text)) return

    if (this.aff.OCONV) text = this.aff.OCONV.match(text)

    if (handled.has(text)) return

    if (
      checkInclusion &&
      iterate(handled).some(prev => lowercase(text).includes(lowercase(prev)))
    ) {
      return
    }

    handled.add(text)

    yield suggestion.replace(text)
  }

  *suggestions(word: string): Iterable<Suggestion> {
    const handled = new Set<string>()

    const [captype, ...variants] = this.aff.casing.corrections(word)

    const handle: Handler = (suggestion: Suggestion, checkInclusion = false) =>
      this.handle(word, captype, handled, suggestion, checkInclusion)

    if (this.aff.FORCEUCASE && captype === CapType.NO) {
      for (const capitalized of this.aff.casing.capitalize(word)) {
        if (this.correct(capitalized)) {
          yield* handle(new Suggestion(capitalized, "forceucase"))
          return
        }
      }
    }

    let goodEditsFound = false

    for (let idx = 0; idx < variants.length; idx++) {
      const variant = variants[idx]

      if (idx > 0 && this.correct(variant)) yield* handle(new Suggestion(variant, "case"))

      let noCompound = false

      for (const suggestion of this.editSuggestions(
        variant,
        handle,
        C.MAX_SUGGESTIONS,
        false
      )) {
        yield suggestion

        goodEditsFound ||= C.GOOD_EDITS.includes(suggestion.kind)

        // prettier-ignore
        switch(suggestion.kind) {
          case "uppercase":
          case "replchars":
          case "mapchars": {
            noCompound = true
            break
          }
          case "spaceword": return
        }
      }

      if (!noCompound) {
        for (const suggestion of this.editSuggestions(
          word,
          handle,
          this.aff.MAXCPDSUGS,
          true
        )) {
          yield suggestion
          goodEditsFound ||= C.GOOD_EDITS.includes(suggestion.kind)
        }
      }

      if (goodEditsFound) return

      if (word.includes("-") && !iterate(handled).some(word => word.includes("-"))) {
        const chunks = word.split("-")

        for (let idx = 0; idx < chunks.length; idx++) {
          const chunk = chunks[idx]
          if (!this.correct(chunk)) {
            for (const suggestion of this.suggestions(chunk)) {
              const candidate = [
                ...chunks.slice(0, idx),
                suggestion.text,
                ...chunks.slice(idx + 1)
              ].join("-")
              if (this.lookup.check(candidate)) yield new Suggestion(candidate, "dashes")
            }
          }
        }
      }

      let ngramsSeen = 0
      for (const suggestion of this.ngramSuggestions(word, handled)) {
        for (const res of handle(new Suggestion(suggestion, "ngram"), true)) {
          ngramsSeen++
          yield res
        }
        if (ngramsSeen >= this.aff.MAXNGRAMSUGS) break
      }

      let phonetSeen = 0
      for (const suggestion of this.phonetSuggestions(word)) {
        for (const res of handle(new Suggestion(suggestion, "phonet"), true)) {
          phonetSeen++
          yield res
        }
        if (phonetSeen >= C.MAX_PHONET_SUGGESTIONS) break
      }
    }
  }

  private *filterSuggestions(
    suggestions: Iterable<Suggestion | MultiWordSuggestion>,
    compounds?: boolean
  ) {
    for (const suggestion of suggestions) {
      if (suggestion instanceof MultiWordSuggestion) {
        if (suggestion.words.every(word => this.correct(word, compounds))) {
          yield suggestion.stringify()
          if (suggestion.allowDash) yield suggestion.stringify("-")
        }
      } else if (this.correct(suggestion.text, compounds)) {
        yield suggestion
      }
    }
  }

  *editSuggestions(word: string, handle: Handler, limit: number, compounds: boolean) {
    let count = 0
    for (const suggestion of this.filterSuggestions(this.edits(word), compounds)) {
      for (const res of handle(suggestion)) {
        yield res
        count++
        if (count > limit) return
      }
    }
  }

  *edits(word: string): Iterable<Suggestion | MultiWordSuggestion> {
    yield new Suggestion(this.aff.casing.upper(word), "uppercase")

    for (const suggestion of replchars(word, this.aff.REP)) {
      if (Array.isArray(suggestion)) {
        yield new Suggestion(suggestion.join(" "), "replchars")
        yield new MultiWordSuggestion(suggestion, "replchars", false)
      } else {
        yield new Suggestion(suggestion, "replchars")
      }
    }

    for (const words of twowords(word)) {
      yield new Suggestion(words.join(" "), "spaceword")
      if (this.dashes) yield new Suggestion(words.join("-"), "spaceword")
    }

    yield* this.editsFrom(mapchars(word, this.aff.MAP), "mapchars")

    yield* this.editsFrom(swapchar(word), "swapchar")

    yield* this.editsFrom(longswapchar(word), "longswapchar")

    yield* this.editsFrom(badcharkey(word, this.aff.KEY), "badcharkey")

    yield* this.editsFrom(badchar(word, this.aff.TRY), "badchar")

    yield* this.editsFrom(doubletwochars(word), "doubletwochars")

    if (!this.aff.NOSPLITSUGS) {
      for (const suggestionPair of twowords(word)) {
        yield new MultiWordSuggestion(suggestionPair, "twowords", this.dashes)
      }
    }
  }

  *editsFrom(iter: Iterable<string>, name: string) {
    for (const suggestion of iter) {
      yield new Suggestion(suggestion, name)
    }
  }

  *ngramSuggestions(word: string, handled: Set<string>) {
    if (this.aff.MAXNGRAMSUGS === 0) return
    yield* ngramSuggest(
      lowercase(word),
      this.ngramWords,
      this.aff.PFX,
      this.aff.SFX,
      iterate(handled).map(lowercase).toSet(),
      this.aff.MAXDIFF,
      this.aff.ONLYMAXDIFF,
      Boolean(this.aff.PHONE)
    )
  }

  *phonetSuggestions(word: string) {
    if (!this.aff.PHONE) return
    yield* phonetSuggest(word, this.ngramWords, this.aff.PHONE)
  }
}
