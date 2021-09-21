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
import type { ChunkBuffer } from "./chunk/buffer"
import type * as DF from "./grammar/definition"
import { Grammar } from "./grammar/grammar"
import { ParserFactory } from "./parser"
import type { ParserConfiguration, TarnationLanguageDefinition } from "./types"
import { makeTopNode } from "./util"

export class TarnationLanguage {
  declare languageData: Record<string, any>
  declare grammarData: DF.Grammar | (() => DF.Grammar)
  declare configure: ParserConfiguration
  declare extensions: Extension[]
  declare description: LanguageDescription
  declare nestLanguages: LanguageDescription[] | Facet<LanguageDescription>

  // only shows up after loading

  declare grammar?: Grammar
  declare top?: NodeType
  declare nodeTypes?: NodeType[]
  declare nodeSet?: NodeSet
  declare stateProp?: NodeProp<ChunkBuffer>
  declare support?: LanguageSupport
  declare language?: Language

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
    supportExtensions = [],
    addToLanguageList = true
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

    if (addToLanguageList) this.extensions.push(addLanguages(this.description))
  }

  /**
   * Loads and processes the language. Calling this function repeatedly
   * will just return the previously loaded language.
   */
  load() {
    // setup grammar data
    if (this.description?.support) return this.description.support
    const def =
      typeof this.grammarData === "function" ? this.grammarData() : this.grammarData
    this.grammar = new Grammar(def, this.configure.variables)

    // merge data from the grammar
    Object.assign(this.languageData, this.grammar.data)

    // setup node data

    this.stateProp = new NodeProp<ChunkBuffer>({ perNode: true })

    const { facet, top } = makeTopNode(this.description.name, this.languageData)
    this.top = top

    const nodeTypes = this.grammar.repository.nodes().map(n => n.type)
    nodeTypes.unshift(NodeType.none, top)

    let nodeSet = new NodeSet(nodeTypes)

    if (this.configure.props) nodeSet = nodeSet.extend(...this.configure.props)

    this.nodeTypes = nodeTypes
    this.nodeSet = nodeSet

    // setup language support
    this.language = new Language(facet, new ParserFactory(this), top)
    this.support = new LanguageSupport(this.language, this.extensions)
    this.loaded = true

    return this.support
  }
}
