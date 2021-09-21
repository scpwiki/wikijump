import { Input, NodeProp, NodeType, Tree } from "@lezer/common"
import type { LRParser, ParserConfig } from "@lezer/lr"
import {
  defineLanguageFacet,
  Extension,
  languageDataProp,
  LanguageDescription,
  LanguageSupport,
  LRLanguage
} from "@wikijump/codemirror/cm"
import { dedupe } from "@wikijump/util"
import type { Regex } from "./grammar/definition"
import type { GrammarToken } from "./types"

export interface CreateLezerLanguageOpts {
  name: string
  parser: LRParser
  configure?: ParserConfig
  alias?: string[]
  ext?: string[]
  languageData?: Record<string, any>
  extensions?: Extension[]
}

export function createLezerLanguage(opts: CreateLezerLanguageOpts) {
  const langDesc = Object.assign(
    { name: opts.name },
    opts.alias ? { alias: opts.alias } : {},
    opts.ext ? { extensions: opts.ext } : {}
  )
  const langData = { ...langDesc, ...(opts.languageData ?? {}) }

  const load = function () {
    const lang = LRLanguage.define({
      parser: opts.parser.configure(opts.configure ?? {}),
      languageData: langData
    })
    return new LanguageSupport(lang, opts.extensions)
  }

  const description = LanguageDescription.of({ ...langDesc, load: async () => load() })

  return { load, description }
}

/** Class that implements the Lezer `Input` interface using a normal string. */
export class StringInput implements Input {
  constructor(readonly string: string) {}

  get length() {
    return this.string.length
  }

  chunk(from: number) {
    return this.string.slice(from)
  }

  readonly lineChunks = false

  read(from: number, to: number) {
    return this.string.slice(from, to)
  }
}

/**
 * Safely compiles a regular expression.
 *
 * @example
 *
 * ```ts
 * // returns null if features aren't supported (e.g. Safari)
 * const regex = re`/(?<=\d)\w+/d`
 * ```
 */
export function re(str: TemplateStringsArray | string, forceFlags = "") {
  const input = typeof str === "string" ? str : str.raw[0]
  const split = /^!?\/([^]+)\/([^]*)$/.exec(input)

  if (!split || !split[1]) return null

  let [, src = "", flags = ""] = split

  if (forceFlags) flags = dedupe([...flags, ...forceFlags]).join("")

  try {
    return new RegExp(src, flags)
  } catch (err) {
    console.warn("cm-tarnation: Recovered from failed RegExp construction")
    console.warn("cm-tarnation: RegExp source:", input)
    console.warn(err)
    return null
  }
}

/**
 * Tests if the given string is a "RegExp string", as in it's in the format
 * of a native `RegExp` statement.
 */
export function isRegExpString(str: string): str is Regex {
  const split = /^!?\/([^]+)\/([^]*)$/.exec(str)
  if (!split || !split[1]) return false
  return true
}

/** Returns if the given `RegExp` has any remembered capturing groups. */
export function hasCapturingGroups(regexp: RegExp) {
  // give an alternative that always matches
  const always = new RegExp(`|${regexp.source}`)
  // ... which means we can use it to get a successful match,
  // regardless of the original regex. this is a bit of a hack,
  // but we can use this to detect capturing groups.
  return always.exec("")!.length > 1
}

/**
 * Creates a lookbehind function from a `RegExp`. This function can only
 * test for a pattern's (non) existence, so no matches or capturing groups
 * are returned.
 *
 * @param pattern - A `RegExp` to be used as a pattern.
 * @param negative - Negates the pattern.
 */
export function createLookbehind(pattern: RegExp, negative?: boolean) {
  // can't be sticky, global, or multiline
  const flags = pattern.flags.replaceAll(/[ygm]/g, "")

  // regexp that can only match at the end of a string
  const regex = new RegExp(`(?:${pattern.source})$`, flags)

  return (str: string, pos: number) => {
    const clipped = str.slice(0, pos)
    const result = regex.test(clipped)
    return negative ? !result : result
  }
}

/**
 * Returns if the `last` `GrammarToken` is effectively equivalent to the
 * `next` `GrammarToken`, as in the two can be merged without any loss of
 * information.
 *
 * @param last - The token to check if it can be potentially extended.
 * @param next - The new token which may be able to merge into the `last` token.
 */
export function canContinue(last?: GrammarToken, next?: GrammarToken) {
  if (!last || !next) return false // tokens are invalid
  // parser directives, or nested language present
  if (last.length > 3 || next.length > 3) return false
  // types aren't equivalent
  if (last[0] !== next[0]) return false
  // tokens aren't inline
  if (last[2] !== next[1]) return false
  // tokens are effectively equivalent
  return true
}

/**
 * Utility for creating a top `NodeType` for a CodeMirror language. Returns
 * both the language's data `Facet` and the `NodeType`.
 *
 * @param name - The name of the language.
 * @param data - The language data.
 */
export function makeTopNode(name: string, data: Record<string, any>) {
  const facet = defineLanguageFacet(data)
  const top = NodeType.define({
    id: 1,
    name,
    top: true,
    props: [[languageDataProp, facet]]
  })
  return { facet, top }
}

/**
 * A special per-node `NodeProp` used for describing nodes where a nested
 * parser will be embedded.
 */
export const EmbeddedParserProp = new NodeProp<string>({ perNode: true })

/** A special `NodeType` used to mark nodes where a nested parser will be embedded. */
export const EmbeddedParserType = NodeType.define({
  id: 2,
  name: "EmbeddedParser"
})

/**
 * Returns a new `Tree` that has been configured as a node that indicates a
 * nested parsing region.
 *
 * @param name - The name of the language.
 * @param from - The start of the region.
 * @param to - The end of the region.
 */
export function getEmbeddedParserNode(name: string, from: number, to: number) {
  return new Tree(EmbeddedParserType, [], [], to - from, [[EmbeddedParserProp, name]])
}

/**
 * Clones any array of arrays. Doesn't clone anything in the nested arrays
 * beyond primitives.
 *
 * @param arr - The nested array to clone.
 */
export function cloneNestedArray<T extends any[][]>(arr: T): T {
  const clone = new Array(arr.length)
  for (let idx = 0; idx < arr.length; idx++) {
    clone[idx] = arr[idx].slice()
  }
  return clone as T
}

export function concatUInt32Arrays(arrays: Uint32Array[]) {
  let total = 0
  for (let i = 0; i < arrays.length; i++) {
    total += arrays[i].length
  }

  const result = new Uint32Array(total)

  let offset = 0
  for (let i = 0; i < arrays.length; i++) {
    result.set(arrays[i], offset)
    offset += arrays[i].length
  }

  return result
}
