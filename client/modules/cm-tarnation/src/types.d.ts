import type { LanguageDescription } from "@codemirror/language"
import type { Extension } from "@codemirror/state"
import type { NodePropSource, NodeType, Tree } from "lezer-tree"
import type * as DF from "./grammar/definition"
import type { Chunk, TokenizerBuffer } from "./tokenizer"

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
   * The grammar that will be used to tokenize the language.
   *
   * This value can be provided as a function, which will cause the grammar
   * to be lazily evaluated.
   */
  grammar: DF.Grammar | (() => DF.Grammar)
  /**
   * A list of `LanguageDescription` objects that will be used when the
   * parser nests in a language.
   */
  nestLanguages?: LanguageDescription[]
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
}

// -- TOKENS

/** Directs the parser to nest tokens using the node's type ID. */
export type MappedParserAction = [id: number, inclusive: number][]

/** A more efficient representation of a `GrammarToken`. */
export type MappedToken = [
  type: number,
  from: number,
  to: number,
  open?: MappedParserAction,
  close?: MappedParserAction
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

/** Represents a node in a tokenizer stack. */
export type TokenizerStackElement = [state: string, context: DF.Context]

/** A serialized (just data) form of a tokenizer stack. */
export interface SerializedTokenizerStack {
  stack: TokenizerStackElement[]
  embedded: null | [lang: string, start: number]
}

/** Serialized context/state of a tokenizer. */
export interface SerializedTokenizerContext {
  pos: number
  stack: SerializedTokenizerStack
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
  embedded: EmbeddedData
}

/** A parser's cache, mapping tokenizer chunks to parser contexts. */
export type ParserCache = WeakMap<Chunk, SerializedParserContext>

// -- MISC.

/**
 * The region of a document that should be parsed, along with other
 * information such as what the edited range of the document was.
 */
export interface ParseRegion {
  /** The parser should start before or at this position. */
  from: number
  /** The parse should stop past or at this position. */
  to: number
  /** The length of the document. */
  length: number
  /** The edited range of the document. */
  edit?: {
    /** The start of the edited range. */
    from: number
    /** The end of the edited range. */
    to: number
    /** The number of characters added in the change. */
    offset: number
  }
}

export type AddNodeSpec = { name: string } & Omit<
  Parameters<typeof NodeType["define"]>[0],
  "id" | "name"
>

/**
 * Describes a Tarnation's language cache, which is how it recovers
 * information for incremental parsing.
 *
 * - `Tree` maps to `TokenizerBuffer`
 * - `Chunk` maps to `SerializedParserContext`
 */
export type CacheMap = WeakMap<Tree | Chunk, TokenizerBuffer | SerializedParserContext>
