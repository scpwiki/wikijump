import type { Input, NodePropSource, TreeCursor } from "@lezer/common"
import type { Extension, Facet, LanguageDescription } from "@wikijump/codemirror/cm"
import type { Grammar } from "./grammar/definition"
import type { Node } from "./grammar/node"
import type { Rule } from "./grammar/rules/rule"
import type { State } from "./grammar/rules/state"

// -- CONFIGURATION

export interface ParserConfiguration {
  /** Node props to add to the emitted nodes of the grammar. */
  props?: NodePropSource[]
  /**
   * A record of variables to pass to the grammar. They can be referenced
   * in the grammar using the `$var:foo` syntax.
   */
  variables?: VariableTable

  /**
   * A special function that can be provided for nesting languages. It is
   * given a node, in the form of a `TreeCursor`, and the document `Input`.
   * It should return `null` (skip this node) or an object with a `name`
   * string and a list of ranges in the `overlay` property.
   *
   * If the latter is returned, that information will be used to nest, if
   * possible, the given language (by name) in the ranges specified.
   */
  nest?: (
    cursor: TreeCursor,
    input: Input
  ) => null | { name: string; overlay: { from: number; to: number }[] }
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
  grammar: Grammar | (() => Grammar)
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

// -- GRAMMAR

/**
 * Standard interface for a matcher of some kind. Takes a string input and
 * matches it against some sort of internal pattern.
 */
export interface Matcher {
  /** Returns if the given string is matched by a pattern. */
  test(str: string, pos?: number): boolean
  /**
   * Returns a {@link MatchOutput}, describing how a string matched against
   * a pattern (if it did at all).
   */
  match(str: string, pos?: number): MatchOutput
}

/** Standard output for a {@link Matcher}. */
export type MatchOutput = null | {
  /** The entirety of the substring matched. */
  total: string
  /**
   * Captures for this match, if any. Captures must be contiguous
   * substrings of the total match.
   */
  captures: string[] | null
  /** The length of the match. */
  length: number
}

/** A variable for use by a {@link Grammar}. */
export type Variable = Matcher | string | string[] | RegExp

/** A simple record of {@link Variable}s. */
export type VariableTable = Record<string, Variable>

/** An individual element in a {@link GrammarStack}. */
export interface GrammarStackElement {
  /** The current parent {@link Node}. */
  node: Node
  /** The rules to loop parsing with. */
  rules: (Rule | State)[]
  /**
   * A specific {@link Rule} that, when matched, should pop this element off
   * the stack.
   */
  end: Rule | State | null
}

/** Represents how the parser should nest tokens. */
export type ParserAction = number[]

// -- TOKENS

/** Token emitted by a {@link Matched} when compiled. */
export type GrammarToken = [
  id: number | null,
  from: number,
  to: number,
  open?: ParserAction,
  close?: ParserAction
]
