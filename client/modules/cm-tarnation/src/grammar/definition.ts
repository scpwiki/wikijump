import type { Tag as cmTag, tags as cmTags } from "@wikijump/codemirror/cm"

// TODO: document this
// adding doc comments would be helpful, but what would be more helpful
// is a JSON schema. doing that will take a while, so for now this is undocumented.
// it's _mostly_ self-explanatory, but there are some things that are not obvious

export interface Grammar {
  // CodeMirror language data

  comments?: {
    block?: { open: string; close: string }
    line?: string
  }

  closeBrackets?: {
    brackets?: string[]
    before?: string
  }

  indentOnInput?: Regex

  wordChars?: string

  // actual grammar

  ignoreCase?: boolean

  default?: Node

  repository?: Record<string, RepositoryItem>

  includes?: Record<string, string[]>

  global?: Inside

  root: Inside
}

export type RepositoryItem = Regex | Node | ReuseNode | Rule | State

export type Rule = Lookup | Pattern | Chain

export interface Node {
  type?: string
  open?: string
  close?: string
  emit?: string | boolean
  nest?: string
  // CodeMirror properties, doesn't affect grammar
  tag?: Tag
  openedBy?: Arrayable<string>
  closedBy?: Arrayable<string>
  group?: Arrayable<string>
  fold?: boolean | "inside" | "past_first_line" | `offset(${number}, ${number})`
  indent?:
    | "flat"
    | `delimited(${string})`
    | "continued"
    | `continued(${Regex})`
    | `add(${number})`
    | `set(${number})`
}

export interface State extends Node {
  begin: string | Rule
  end: string | Rule
  inside?: Inside | Node | ReuseNode | "inherit" | "loose"
}

export interface RuleOptions extends Node {
  captures?: Record<string, Node | ReuseNode | CaptureCondition>
  context?: Arrayable<ContextSetter>
  contextImmediate?: boolean
  lookbehind?: LookbehindSource
  lookahead?: Regex
  rematch?: boolean
}

export interface Lookup extends RuleOptions {
  lookup: string[] | VarIndex
}

export interface Pattern extends RuleOptions {
  match: Arrayable<string | Regex | VarIndex>
}

export interface Chain extends RuleOptions {
  chain: string[]
  skip?: Regex
}

export interface ContextSetter {
  if?: MatchIndex
  matches?: string | Regex | VarIndex
  set: string
  to: string | MatchIndex | null
}

export interface CaptureCondition {
  if?: MatchIndex
  matches: string | Regex | VarIndex | MatchIndex
  then?: Node | ReuseNode
  else?: Node | ReuseNode
}

export type ReuseNode = { is: string }

export type Inside = (string | Rule | Include | State)[]

export type Include = { include: string }

export type StyleTag = keyof FilterOut<typeof cmTags, (tag: cmTag) => cmTag>
export type FunctionTag = keyof FilterFor<typeof cmTags, (tag: cmTag) => cmTag>
export type TagModifier = `(${`${string}/` | "!" | "..."}) ` | ""
export type TagFunction = `${FunctionTag}(${StyleTag})`
export type Tag = `${TagModifier}${string}`

export type ChainItem = `${string}${"?" | "*" | "+" | ""}`

export type Regex = `/${string}/${string}`
export type LookbehindSource = `${"!" | ""}${Regex}`

export type MatchIndex = `$${number}`
export type VarIndex = `$var:${string}`
export type ContextIndex = `$ctx:${string}`
