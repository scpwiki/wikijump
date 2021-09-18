import type { NodePropSource, NodeType, Tree } from "@lezer/common"
import type { Extension, Facet, LanguageDescription } from "@wikijump/codemirror/cm"
import type * as DF from "./grammar/definition"
import type { ParserAction, VariableTable } from "./grammar/types"
import type { Chunk } from "./tokenizer"

// -- CONFIGURATION

export interface ParserConfiguration {
  props?: NodePropSource[]
}

/** The options / interface required to create a Tarnation language. */
export interface TarnationLanguageDefinition {
  /**
   * The name of the language. This property is important for CodeMirror,
   * so make sure it's reasonable.
   */
  name: string
  /**
   * A record of variables to pass to the grammar. They can be referenced
   * in the grammar using the `$var:foo` syntax.
   */
  variables?: VariableTable
  /**
   * The grammar that will be used to tokenize the language.
   *
   * This value can be provided as a function, which will cause the grammar
   * to be lazily evaluated.
   */
  grammar: DF.Grammar | (() => DF.Grammar)
  /**
   * A list (or facet) of `LanguageDescription` objects that will be used
   * when the parser nests in a language.
   */
  nestLanguages?: LanguageDescription[] | Facet<LanguageDescription>
  /** Configuration options for the parser, such as node props. */
  configure?: ParserConfiguration
  /** A list of aliases for the name of the language. (e.g. 'go' - `['golang']`) */
  alias?: string[]
  /** A list of file extensions. (e.g. `['.ts']`) */
  extensions?: string[]
  /**
   * The 'languageData' field inherit to the {@link Language}. CodeMirror
   * plugins are defined by, or use, the data in this field. e.g.
   * indentation, autocomplete, etc.
   */
  languageData?: Record<string, any>
  /** Extra extensions to be loaded. */
  supportExtensions?: Extension[]
  /**
   * If true, this language will be automatically added to the
   * {@link languageList} facet. Defaults to true.
   */
  addToLanguageList?: boolean
}

// -- TOKENS

/** A more efficient representation of a `GrammarToken`. */
export type MappedToken = [
  type: number | null,
  from: number,
  to: number,
  open?: ParserAction,
  close?: ParserAction
]

/** Represents the region of an embedded language. */
export type EmbedToken = [lang: string, from: number, to: number]

export type Token = MappedToken | EmbedToken

/**
 * Represents a Lezer token. The `tree` value is for storing a reusable
 * form of this token and its children.
 */
type LezerToken = [id: number, from: number, to: number, children: number, tree?: Tree]

// -- TOKENIZER

/** Serialized context/state of a tokenizer. */
export interface SerializedTokenizerContext {
  pos: number
  state: GrammarState
  embedded: null | [lang: string, start: number]
}

// -- PARSER

/** Stack used by the parser to track tree construction. */
export type ParserElementStack = [name: number, start: number, children: number][]

/** Embedded region data for a parser. */
export type EmbeddedData = [token: LezerToken, language: string][]

/** Serialized context/state of a parser. */
export interface SerializedParserContext {
  pos: number
  index: number
  buffer: LezerToken[]
  stack: ParserElementStack
}

/** A parser's cache, mapping tokenizer chunks to parser contexts. */
export type ParserCache = WeakMap<Chunk, SerializedParserContext>

// -- MISC.

export type AddNodeSpec = { name: string } & Omit<
  Parameters<typeof NodeType["define"]>[0],
  "id" | "name"
>
