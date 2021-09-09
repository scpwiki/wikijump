import { NodeProp, NodeType } from "@lezer/common"
import { isFunction } from "is-what"
import {
  Extension,
  Language,
  LanguageDescription,
  LanguageSupport
} from "wj-codemirror/cm"
import { removeUndefined } from "wj-util"
import { DelegatorFactory } from "./delegator"
import type * as DF from "./grammar/definition"
import { Grammar } from "./grammar/grammar"
import { NodeMap } from "./node-map"
import { TokenizerBuffer } from "./tokenizer"
import type { ParserConfiguration, TarnationLanguageDefinition } from "./types"
import { EmbeddedParserType, makeTopNode } from "./util"

export class TarnationLanguage {
  private declare languageData: Record<string, any>
  private declare grammarData: DF.Grammar | (() => DF.Grammar)
  private declare configure: ParserConfiguration
  private declare extensions: Extension[]

  declare description: LanguageDescription
  declare grammar?: Grammar
  declare nodes?: NodeMap
  declare top?: NodeType
  declare stateProp?: NodeProp<TokenizerBuffer>
  declare support?: LanguageSupport
  declare language?: Language
  declare nestLanguages: LanguageDescription[]

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
    // setup grammar data
    if (this.description?.support) return this.description.support
    const def = isFunction(this.grammarData) ? this.grammarData() : this.grammarData
    this.grammar = new Grammar(def)

    // setup node data
    const nodes = (this.nodes = new NodeMap())

    this.stateProp = new NodeProp<TokenizerBuffer>({ perNode: true })

    const { facet, top } = makeTopNode(this.description.name, this.languageData)
    this.top = top

    nodes.add(NodeType.none, "None")
    nodes.add(top, "Document")
    nodes.add(EmbeddedParserType, "EmbeddedParser")

    this.grammar.types.forEach(name => nodes.add({ name }))

    if (this.grammar.props.length) nodes.configure({ props: this.grammar.props })
    if (this.configure.props) nodes.configure(this.configure)

    // setup language support
    this.language = new Language(facet, new DelegatorFactory(this), top)
    this.support = new LanguageSupport(this.language, this.extensions)
    this.loaded = true

    return this.support
  }
}
