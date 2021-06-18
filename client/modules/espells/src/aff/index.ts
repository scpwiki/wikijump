import iterate from "iterare"
import type { Reader } from "../reader"
import { Trie } from "../trie"
import { escapeRegExp, re, reverse } from "../util"
import { Prefix, Suffix } from "./affix"
import { Casing, GermanCasing, TurkicCasing } from "./casing"
import { CompoundPattern } from "./compound-pattern"
import { CompoundRule } from "./compound-rule"
import { ConvTable } from "./conv-table"
import { PhonetTable } from "./phonet-table"
import { RepPattern } from "./rep-pattern"

const SYNONYMS: Record<string, string> = {
  PSEUDOROOT: "NEEDAFFIX",
  COMPOUNDLAST: "COMPOUNDEND"
}

const FLAG_LONG_REGEXP = /../
const FLAG_NUM_REGEXP = /\d+(?=,|$)/

export class Aff {
  SET = "UTF-8" // unused
  FLAG: "short" | "long" | "numeric" | "UTF-8" = "short"
  LANG?: string
  WORDCHARS?: string // unused
  IGNORE?: Set<string>
  CHECKSHARPS = false
  FORBIDDENWORD?: string

  KEY = "qwertyuiop|asdfghjkl|zxcvbnm"
  TRY = ""
  NOSUGGEST?: string
  KEEPCASE?: string
  REP: Set<RepPattern> = new Set()
  MAP: Set<Set<string>> = new Set()
  NOSPLITSUGS = false
  PHONE?: PhonetTable
  MAXCPDSUGS = 3
  MAXNGRAMSUGS = 4
  MAXDIFF = 1
  ONLYMAXDIFF = false

  PFX: Map<string, Set<Prefix>> = new Map()
  SFX: Map<string, Set<Suffix>> = new Map()
  NEEDAFFIX?: string
  CIRCUMFIX?: string
  COMPLEXPREFIXES = false
  FULLSTRIP = false

  BREAK: Set<RegExp> = new Set([/-/g, /^-/g, /-$/g])
  COMPOUNDRULE: Set<CompoundRule> = new Set()
  COMPOUNDMIN = 3
  COMPOUNDWORDMAX?: number
  COMPOUNDFLAG?: string
  COMPOUNDBEGIN?: string
  COMPOUNDMIDDLE?: string
  COMPOUNDEND?: string
  ONLYINCOMPOUND?: string
  COMPOUNDPERMITFLAG?: string
  COMPOUNDFORBIDFLAG?: string
  FORCEUCASE?: string
  CHECKCOMPOUNDCASE = false
  CHECKCOMPOUNDUP = false
  CHECKCOMPOUNDREP = false
  CHECKCOMPOUNDTRIPLE = false
  CHECKCOMPOUNDPATTERN: Set<CompoundPattern> = new Set()
  SIMPLIFIEDTRIPLE = false
  COMPOUNDSYLLABLE?: [number, string] // unused
  COMPOUNDMORESUFFIXES = false // unused
  COMPOUNDROOT?: string // unused

  ICONV?: ConvTable
  OCONV?: ConvTable

  AF: Set<string>[] = []
  AM: Set<string>[] = []

  WARN?: string
  FORBIDWARN = false
  SYLLABLENUM?: string // unused
  SUBSTANDARD?: string // unused

  declare casing: Casing
  declare prefixesIndex: Trie<Set<Prefix>>
  declare suffixesIndex: Trie<Set<Suffix>>

  constructor(reader: Reader) {
    do {
      if (reader.done) break

      let [directive, ...args] = reader.line.split(/\s+/u)

      // skip directive if it doesn't seem real
      if (!/^[A-Z]+$/.test(directive)) continue

      if (SYNONYMS.hasOwnProperty(directive)) directive = SYNONYMS[directive]

      switch (directive) {
        case "SET":
        case "FLAG":
        case "KEY":
        case "TRY":
        case "WORDCHARS":
        case "LANG": {
          // @ts-ignore
          this[directive] = args[0]
          break
        }

        case "IGNORE": {
          this.IGNORE = new Set([...args])
          break
        }

        case "MAXDIFF":
        case "MAXNGRAMSUGS":
        case "MAXCPDSUGS":
        case "COMPOUNDMIN":
        case "COMPOUNDWORDMAX": {
          this[directive] = parseInt(args[0])
          break
        }

        case "NOSUGGEST":
        case "KEEPCASE":
        case "CIRCUMFIX":
        case "NEEDAFFIX":
        case "FORBIDDENWORD":
        case "WARN":
        case "COMPOUNDFLAG":
        case "COMPOUNDBEGIN":
        case "COMPOUNDMIDDLE":
        case "COMPOUNDEND":
        case "ONLYINCOMPOUND":
        case "COMPOUNDPERMITFLAG":
        case "COMPOUNDFORBIDFLAG":
        case "FORCEUCASE":
        case "SUBSTANDARD":
        case "SYLLABLENUM":
        case "COMPOUNDROOT": {
          // @ts-ignore
          this[directive] = this.parseFlag(args[0])
          break
        }

        case "COMPLEXPREFIXES":
        case "FULLSTRIP":
        case "NOSPLITSUGS":
        case "CHECKSHARPS":
        case "CHECKCOMPOUNDCASE":
        case "CHECKCOMPOUNDUP":
        case "CHECKCOMPOUNDREP":
        case "CHECKCOMPOUNDTRIPLE":
        case "SIMPLIFIEDTRIPLE":
        case "ONLYMAXDIFF":
        case "COMPOUNDMORESUFFIXES":
        case "FORBIDWARN": {
          this[directive] = true
          break
        }

        case "BREAK": {
          reader.for(parseInt(args[0]), line => {
            let [, pattern] = line.split(/\s+/u)
            pattern = escapeRegExp(pattern).replaceAll("\\^", "^").replaceAll("\\$", "$")
            this.BREAK.add(re`/${pattern}/g`)
          })
          break
        }

        case "COMPOUNDRULE": {
          reader.for(parseInt(args[0]), line => {
            const [, value] = line.split(/\s+/u)
            this.COMPOUNDRULE.add(new CompoundRule(value, this))
          })
          break
        }

        case "ICONV":
        case "OCONV": {
          const pairs: [string, string][] = []
          reader.for(parseInt(args[0]), line => {
            const [, pattern, replacement] = line.split(/\s+/u)
            pairs.push([pattern, replacement])
          })
          this[directive] = new ConvTable(pairs)
          break
        }

        case "REP": {
          reader.for(parseInt(args[0]), line => {
            const [, pattern, replacement] = line.split(/\s+/u)
            this.REP.add(new RepPattern(pattern, replacement))
          })
          break
        }

        case "MAP": {
          reader.for(parseInt(args[0]), line => {
            const [, value] = line.split(/\s+/u)
            this.MAP.add(
              iterate(value.matchAll(/\(.*?\)|./g))
                .map(match => match[0].replaceAll(/^\(|\)$/g, ""))
                .toSet()
            )
          })
          break
        }

        case "PFX": {
          const [flag, crossproduct, count] = args
          reader.for(parseInt(count), line => {
            const [, , strip, add, cond] = line.split(/\s+/u)
            const prefix = new Prefix(flag, crossproduct, strip, add, cond, this)
            const set = this.PFX.get(flag) ?? new Set()
            this.PFX.set(flag, set.add(prefix))
          })
          break
        }

        case "SFX": {
          const [flag, crossproduct, count] = args
          reader.for(parseInt(count), line => {
            const [, , strip, add, cond] = line.split(/\s+/u)
            const suffix = new Suffix(flag, crossproduct, strip, add, cond, this)
            const set = this.SFX.get(flag) ?? new Set()
            this.SFX.set(flag, set.add(suffix))
          })
          break
        }

        case "CHECKCOMPOUNDPATTERN": {
          reader.for(parseInt(args[0]), line => {
            const [, left, right, replacement] = line.split(/\s+/u)
            this.CHECKCOMPOUNDPATTERN.add(new CompoundPattern(left, right, replacement))
          })
          break
        }

        case "AF": {
          reader.for(parseInt(args[0]), line => {
            const [, value] = line.split(/\s+/u)
            this.AF.push(this.parseFlags(value))
          })
          break
        }

        case "AM": {
          reader.for(parseInt(args[0]), line => {
            const [, value] = line.split(/\s+/u)
            this.AM.push(new Set<string>(value.split("")))
          })
        }

        case "COMPOUNDSYLLABLE": {
          this.COMPOUNDSYLLABLE = [parseInt(args[0]), args[1]]
          break
        }

        case "PHONE": {
          const rules: [string, string][] = []
          reader.for(parseInt(args[0]), line => {
            const [, search, replacement] = line.split(/\s+/u)
            rules.push([search, replacement])
          })
          this.PHONE = new PhonetTable(rules)
          break
        }
      }
    } while (reader.next())

    if (this.CHECKSHARPS) {
      this.casing = new GermanCasing()
    } else if (this.LANG && ["tr", "tr_TR", "az", "crh"].includes(this.LANG)) {
      this.casing = new TurkicCasing()
    } else {
      this.casing = new Casing()
    }

    this.prefixesIndex = new Trie()
    for (const [, prefixes] of this.PFX) {
      for (const prefix of prefixes) {
        this.prefixesIndex.add(prefix.add, set =>
          !set ? new Set([prefix]) : set.add(prefix)
        )
      }
    }

    this.suffixesIndex = new Trie()
    for (const [, suffixes] of this.SFX) {
      for (const suffix of suffixes) {
        this.suffixesIndex.add(reverse(suffix.add), set =>
          !set ? new Set([suffix]) : set.add(suffix)
        )
      }
    }
  }

  parseFlag(flag: string) {
    return [...this.parseFlags(flag)][0]
  }

  parseFlags(flags: string | string[]) {
    if (typeof flags === "string") flags = [flags]

    const result = flags.flatMap(flag => {
      if (this.AF.length && this.AF[parseInt(flag) - 1]) {
        return [...this.AF[parseInt(flag) - 1]]
      }

      // prettier-ignore
      switch (this.FLAG) {
        case "UTF-8":
        case "short": return [...flag]
        case "long": return FLAG_LONG_REGEXP.exec(flag)?.slice(1) ?? []
        case "numeric": return FLAG_NUM_REGEXP.exec(flag)?.slice(1) ?? []
      }
    })

    return new Set(result)
  }
}
