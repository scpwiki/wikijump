import type { Node } from "./node"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"

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

/** Token emitted by a {@link Matched} when compiled. */
export type GrammarToken = [
  id: number | null,
  from: number,
  to: number,
  open?: ParserAction,
  close?: ParserAction,
  nest?: string | Nesting
]

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
  end: Rule | null
}

/** Represents how the parser should nest tokens. */
export type ParserAction = [number, Inclusivity][]

export enum Nesting {
  /** Special value which indicates that a nested region should be ended. */
  POP
}

export enum Inclusivity {
  /** The token should be excluded from the parent node. */
  EXCLUSIVE,
  /** The token should be included in the parent node. */
  INCLUSIVE
}

export enum Wrapping {
  /** The {@link Node} in this match contains the entirety of the branch. */
  FULL,
  /** The {@link Node} in this match begins the branch. */
  BEGIN,
  /** The {@link Node} in this match ends the branch. */
  END
}
