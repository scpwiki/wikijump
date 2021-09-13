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
