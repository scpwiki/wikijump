import iterate from "iterare"
import type { Aff } from "../aff"
import { CapType } from "../aff/casing"
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

const MAXPHONSUGS = 2
const MAXSUGGESTIONS = 15
const GOODEDITS = ["spaceword", "uppercase", "replchars"]

type Handler = (suggestion: Suggestion, checkInclusion?: boolean) => Generator<Suggestion>

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
    return this.lookup.correct(word, false, false, compounds)
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

  *suggestions(word: string): Generator<Suggestion> {
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
        MAXSUGGESTIONS,
        false
      )) {
        yield suggestion

        goodEditsFound ||= GOODEDITS.includes(suggestion.kind)

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
          goodEditsFound ||= GOODEDITS.includes(suggestion.kind)
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
              if (this.lookup.test(candidate)) yield new Suggestion(candidate, "dashes")
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
        if (phonetSeen >= MAXPHONSUGS) break
      }
    }
  }

  private *filterSuggestions(
    suggestions:
      | Set<Suggestion | MultiWordSuggestion>
      | Generator<Suggestion | MultiWordSuggestion>,
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

  *edits(word: string): Generator<Suggestion | MultiWordSuggestion> {
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

    for (const suggestion of mapchars(word, this.aff.MAP)) {
      yield new Suggestion(suggestion, "swapchar")
    }

    for (const suggestion of swapchar(word)) {
      yield new Suggestion(suggestion, "swapchar")
    }

    for (const suggestion of longswapchar(word)) {
      yield new Suggestion(suggestion, "longswapchar")
    }

    for (const suggestion of badcharkey(word, this.aff.KEY)) {
      yield new Suggestion(suggestion, "badcharkey")
    }

    for (const suggestion of badchar(word, this.aff.TRY)) {
      yield new Suggestion(suggestion, "badchar")
    }

    for (const suggestion of doubletwochars(word)) {
      yield new Suggestion(suggestion, "doubletwochars")
    }

    if (!this.aff.NOSPLITSUGS) {
      for (const suggestionPair of twowords(word)) {
        yield new MultiWordSuggestion(suggestionPair, "twowords", this.dashes)
      }
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
