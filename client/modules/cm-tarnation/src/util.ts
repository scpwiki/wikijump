import { Input, NodeProp, NodeType, Tree } from "@lezer/common"
import type { LRParser, ParserConfig } from "@lezer/lr"
import {
  defineLanguageFacet,
  Extension,
  languageDataProp,
  LanguageDescription,
  LanguageSupport,
  LRLanguage
} from "wj-codemirror/cm"

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

export const EmbeddedParserProp = new NodeProp<string>({ perNode: true })

export const EmbeddedParserType = NodeType.define({
  id: 2,
  name: "EmbeddedParser"
})

export function getEmbeddedParserNode(name: string, from: number, to: number) {
  return new Tree(EmbeddedParserType, [], [], to - from, [[EmbeddedParserProp, name]])
}
