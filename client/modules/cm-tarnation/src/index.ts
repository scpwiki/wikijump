import { styleTags, Tag, tags } from "@codemirror/highlight"
import {
  defineLanguageFacet,
  EditorParseContext,
  Language,
  languageDataProp,
  LanguageDescription,
  LanguageSupport
} from "@codemirror/language"
import type { Extension } from "@codemirror/state"
import { isFunction } from "is-what"
import { Input, NodeProp, NodePropSource, NodeSet, NodeType, Tree } from "lezer-tree"
import { removeUndefined } from "wj-util"
import { Buffer, BufferCache } from "./buffer"
import type * as DF from "./grammar/definition"
import { Grammar } from "./grammar/grammar"
import { Parser } from "./parser"
import { Tokenizer } from "./tokenizer"

export * from "./grammar/helpers"

// TODO: reuse ahead buffer rather than just always viewport skipping
// TODO: fix string with variable array expansion (convert to array of codepoints)
// TODO: add substitution to next, switchto
// TODO: better document the grammar classes
// TODO: better document the parser classes

type AddNodeSpec = { name: string } & Omit<
  Parameters<typeof NodeType["define"]>[0],
  "id" | "name"
>

interface ParserConfiguration {
  props?: NodePropSource[]
}

/** The options / interface required to create a Tarnation language. */
export interface TarnationLanguageDefinition {
  /**
   * The name of the language.
   * This property is important for CodeMirror, so make sure it's reasonable.
   */
  name: string
  /**
   * The grammar that will be used to tokenize the language.
   *
   * This value can be provided as a function,
   * which will cause the grammar to be lazily evaluated.
   */
  grammar: DF.Grammar | (() => DF.Grammar)
  /**
   * A list of `LanguageDescription` objects that will
   * be used when the parser nests in a language.
   */
  nestLanguages?: LanguageDescription[]
  /** Configuration options for the parser, such as node props. */
  configure?: ParserConfiguration
  /** A list of aliases for the name of the language. (e.g. 'go' - `['golang']`) */
  alias?: string[]
  /** A list of file extensions. (e.g. `['.ts']`) */
  extensions?: string[]
  /**
   * The 'languageData' field inherit to the {@link Language}.
   * CodeMirror plugins are defined by, or use, the data in this field.
   * e.g. indentation, autocomplete, etc.
   */
  languageData?: Record<string, any>
  /** Extra extensions to be loaded. */
  supportExtensions?: Extension[]
}

/**
 * Global handler for a Tarnation language.
 * The language constructed will not be processed until the `load` function is called.
 * @see {@link TarnationLanguageDefinition}
 */
export class TarnationLanguage {
  declare description: LanguageDescription
  declare grammar?: Grammar
  declare nodes?: NodeMap
  declare support?: LanguageSupport
  declare language?: Language
  declare state?: State

  declare nestLanguages: LanguageDescription[]

  private declare languageData: Record<string, any>
  private declare grammarData: DF.Grammar | (() => DF.Grammar)
  private declare configure: ParserConfiguration
  private declare extensions: Extension[]

  loaded = false

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
   * Loads and processes the language.
   * Calling this function repeatedly will
   * just return the previously loaded language.
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

    const state = (this.state = new State(this))

    const parser = {
      startParse(input: Input, pos: number, context: EditorParseContext) {
        return new Parser(state, input, pos, context)
      }
    }

    this.language = new Language(facet, parser, topNode)
    this.support = new LanguageSupport(this.language, this.extensions)

    this.loaded = true

    console.log(this)

    return this.support
  }
}

export class State {
  cache = new BufferCache()
  tokenizer = new Tokenizer(this)

  constructor(public language: TarnationLanguage) {}

  get grammar() {
    return this.language.grammar!
  }
  get nodes() {
    return this.language.nodes!
  }
  get nestLanguages() {
    return this.language.nestLanguages
  }

  /** Utility for building a {@link Tree}. */
  buildTree(nodeBuffer: Buffer, start?: number, length?: number) {
    const { buffer, reused } = nodeBuffer.compile()
    const tree = Tree.build({
      topID: 0,
      nodeSet: this.nodes.set,
      buffer,
      reused,
      length,
      start
    })
    this.cache.attach(nodeBuffer, tree)
    return tree
  }
}

export class NodeMap {
  map = new Map<string, number>()
  types: NodeType[] = []
  set = new NodeSet(this.types)

  // @ts-ignore
  private tags: Record<string, Tag> = { ...tags }

  get(name: string) {
    return this.map.get(name)
  }

  add(spec: AddNodeSpec, name?: string): NodeType | null
  add(spec: NodeType, name: string): NodeType | null
  add(spec: AddNodeSpec | NodeType, name?: string): NodeType | null {
    const { map, types, tags } = this
    if (spec instanceof NodeType && name) {
      map.set(name, spec.id)
      types.push(spec)
      return spec
    }
    if (!(spec instanceof NodeType)) {
      /*
       * There is two ways a node can be interpreted:
       * 1. The node's name is lowercased. That means it is a shortcut for a
       *    CodeMirror highlighting tag.
       * 2. The node's name is capitalized. That means it is a custom name,
       *    and no assumptions will be made about its highlighting or styling.
       */
      const id = map.size
      const props: (NodePropSource | [NodeProp<any>, any])[] = [...(spec.props ?? [])]
      if (spec.name && spec.name[0].toUpperCase() !== spec.name[0]) {
        props.push(styleTags({ [`${spec.name}/...`]: tags[spec.name] ?? NodeType.none }))
      }
      const node = NodeType.define({ ...spec, name: spec.name || undefined, id, props })
      map.set(name ?? spec.name, id)
      types.push(node)
      return node
    }
    return null
  }

  configure(config: ParserConfiguration) {
    if ("props" in config) this.set = this.set.extend(...config.props!)
  }
}
