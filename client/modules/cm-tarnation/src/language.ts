import { Input, NodeType } from "@lezer/common"
import { isFunction } from "is-what"
import {
  defineLanguageFacet,
  EditorParseContext,
  Extension,
  Language,
  languageDataProp,
  LanguageDescription,
  LanguageSupport
} from "wj-codemirror/cm"
import { removeUndefined } from "wj-util"
import { Cache } from "./cache"
import type * as DF from "./grammar/definition"
import { Grammar } from "./grammar/grammar"
import { Host } from "./host"
import { NodeMap } from "./node-map"
import type { ParserConfiguration, TarnationLanguageDefinition } from "./types"

export class TarnationLanguage {
  private declare languageData: Record<string, any>
  private declare grammarData: DF.Grammar | (() => DF.Grammar)
  private declare configure: ParserConfiguration
  private declare extensions: Extension[]

  declare description: LanguageDescription
  declare grammar?: Grammar
  declare nodes?: NodeMap
  declare support?: LanguageSupport
  declare language?: Language
  declare nestLanguages: LanguageDescription[]

  cache = new Cache()
  loaded = false
  performance = 0

  constructor({
    name,
    grammar,
    nestLanguages = [],
    configure = {},
    alias,
    extensions,
    languageData = {},
    supportExtensions = []
  }: TarnationLanguageDefinition) {
    const dataDescription = removeUndefined({ name, alias, extensions })

    this.languageData = { ...dataDescription, ...languageData }
    this.nestLanguages = nestLanguages
    this.grammarData = grammar
    this.configure = configure
    this.extensions = supportExtensions

    this.description = LanguageDescription.of({
      ...dataDescription,
      load: async () => this.load()
    })
  }

  /**
   * Loads and processes the language. Calling this function repeatedly
   * will just return the previously loaded language.
   */
  load() {
    if (this.description?.support) return this.description.support
    const def = isFunction(this.grammarData) ? this.grammarData() : this.grammarData
    this.grammar = new Grammar(def)

    const facet = defineLanguageFacet(this.languageData)
    const facetProp = languageDataProp.set(Object.create(null), facet)

    const nodes = (this.nodes = new NodeMap())
    const topNode = nodes.add(
      // @ts-ignore
      new NodeType(this.description.name, facetProp, 0, 1),
      "Document"
    )!

    this.grammar.types.forEach(name => nodes.add({ name }))

    if (this.grammar.props.length) {
      nodes.configure({ props: this.grammar.props })
    }

    if (this.configure.props) {
      nodes.configure(this.configure)
    }

    const startParse = (input: Input, pos: number, context: EditorParseContext) => {
      return new Host(this, input, pos, context)
    }

    this.language = new Language(facet, { startParse }, topNode)
    this.support = new LanguageSupport(this.language, this.extensions)

    this.loaded = true

    return this.support
  }
}
