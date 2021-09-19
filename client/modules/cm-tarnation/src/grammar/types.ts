import type { Node } from "./node"
import type { Rule } from "./rules/rule"
import type { State } from "./rules/state"

export interface Matcher {
  test(str: string, pos?: number): boolean
  match(str: string, pos?: number): MatchOutput
}

export type MatchOutput = null | {
  total: string
  captures: string[] | null
  length: number
}

export type CaptureMap = Record<number, Node>

export type Variable = Matcher | string | string[] | RegExp
export type VariableTable = Record<string, Variable>

export type GrammarToken = [
  id: number | null,
  from: number,
  to: number,
  open?: ParserAction,
  close?: ParserAction,
  nest?: string | Nesting
]

export interface GrammarStackElement {
  node: Node
  rules: (Rule | State)[]
  end: Rule | null
}

export type ParserAction = [number, Inclusivity][]

export enum Nesting {
  POP
}

export enum Inclusivity {
  EXCLUSIVE,
  INCLUSIVE
}

export enum Wrapping {
  FULL,
  BEGIN,
  END
}
