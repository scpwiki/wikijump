import { NodeProp, NodeSet, NodeType } from "@lezer/common"
import { addLanguages } from "@wikijump/codemirror"
import {
  Extension,
  Facet,
  Language,
  LanguageDescription,
  LanguageSupport
} from "@wikijump/codemirror/cm"
import { removeUndefined } from "@wikijump/util"
import { isFunction } from "is-what"
import { DelegatorFactory } from "./delegator"
import type * as DF from "./grammar/definition"
import { Grammar } from "./grammar/grammar"
import type { VariableTable } from "./grammar/types"
import type { TokenizerBuffer } from "./tokenizer"
import type { ParserConfiguration, TarnationLanguageDefinition } from "./types"
import { EmbeddedParserType, makeTopNode } from "./util"

export class TarnationLanguage {
  private declare languageData: Record<string, any>
  private declare grammarData: DF.Grammar | (() => DF.Grammar)
  private declare configure: ParserConfiguration
  private declare extensions: Extension[]

  declare description: LanguageDescription
  declare variables: VariableTable
  declare grammar?: Grammar
  declare top?: NodeType
  declare nodeTypes?: NodeType[]
  declare nodeSet?: NodeSet
  declare stateProp?: NodeProp<TokenizerBuffer>
  declare support?: LanguageSupport
  declare language?: Language
  declare nestLanguages: LanguageDescription[] | Facet<LanguageDescription>

  loaded = false
  performance = 0

  constructor({
    name,
    variables = {},
    grammar,
    nestLanguages = [],
    configure = {},
    alias,
    extensions,
    languageData = {},
    supportExtensions = [],
    addToLanguageList = true
  }: TarnationLanguageDefinition) {
    const dataDescription = removeUndefined({ name, alias, extensions })

    this.languageData = { ...dataDescription, ...languageData }
    this.nestLanguages = nestLanguages
    this.variables = variables
    this.grammarData = grammar
    this.configure = configure
    this.extensions = supportExtensions

    this.description = LanguageDescription.of({
      ...dataDescription,
      load: async () => this.load()
    })

    if (addToLanguageList) this.extensions.push(addLanguages(this.description))
  }

  /**
   * Loads and processes the language. Calling this function repeatedly
   * will just return the previously loaded language.
   */
  load() {
    // setup grammar data
    if (this.description?.support) return this.description.support
    const def = isFunction(this.grammarData) ? this.grammarData() : this.grammarData
    this.grammar = new Grammar(def, this.variables)

    // merge data from the grammar
    Object.assign(this.languageData, this.grammar.data)

    // setup node data

    this.stateProp = new NodeProp<TokenizerBuffer>({ perNode: true })

    const { facet, top } = makeTopNode(this.description.name, this.languageData)
    this.top = top

    const nodeTypes = this.grammar.repository.nodes().map(n => n.type)
    nodeTypes.unshift(NodeType.none, top, EmbeddedParserType)

    let nodeSet = new NodeSet(nodeTypes)

    if (this.configure.props) nodeSet = nodeSet.extend(...this.configure.props)

    this.nodeTypes = nodeTypes
    this.nodeSet = nodeSet

    // setup language support
    this.language = new Language(facet, new DelegatorFactory(this), top)
    this.support = new LanguageSupport(this.language, this.extensions)
    this.loaded = true

    return this.support
  }
}
