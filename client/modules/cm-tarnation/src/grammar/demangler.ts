import { isArray, isObjectLike, isString } from "is-what"
import { klona } from "klona"
import { has, hasSigil, removeUndefined, unSigil } from "wj-util"
import type * as DF from "./definition"

export type ParserAction = [type: string, inclusive: number][]

export interface Grammar {
  fallback?: Action
  ignoreCase?: boolean
  brackets?: DF.Bracket[]
  variables?: Record<string, DF.Variable>
  start?: string
  global?: (DF.Directive | Rule | RuleState)[]
  states: Record<string, (DF.Directive | Rule | RuleState)[]>
}

export interface Rule {
  target?: DF.SubRuleTarget
  match?: DF.Match
  action?: Action
}

export interface RuleState {
  begin: Rule | RuleState
  end: Rule
  type?: string
  embedded?: `${DF.Substitute | string}!`
  rules?: string | (DF.Directive | Rule | RuleState)[]
}

export interface Action {
  type?: string
  group?: Action[]
  next?: string
  open?: ParserAction
  close?: ParserAction
  switchTo?: string
  embedded?: string
  context?: string
  log?: string
  repeat?: boolean
  optional?: boolean
  all?: boolean
  strict?: boolean
  rules?: string | (DF.Directive | Rule | RuleState)[]
}

type RuleDefs = DF.Directive | DF.SubRule | DF.Rule | DF.RuleState

export function demangleGrammar(grammar: DF.Grammar): Grammar {
  const { variables, start, ignoreCase, global, fallback, brackets, states } = grammar

  // demangle states
  const newStates: Record<string, (DF.Directive | Rule | RuleState)[]> = {}
  if (states) {
    for (const state in states) {
      newStates[state] = states[state].map(rule => demangleRule(rule))
    }
  }

  // merge what we can
  const demangled: Grammar = { variables, start, ignoreCase, brackets, states: newStates }

  if (fallback) demangled.fallback = demangleAction(fallback)
  if (global) demangled.global = global.map(rule => demangleRule(rule))

  return removeUndefined(demangled)
}

export function demangleRule(rule: RuleDefs): DF.Directive | Rule | RuleState {
  if (
    "include" in rule ||
    "props" in rule ||
    "style" in rule ||
    "brackets" in rule ||
    "variables" in rule
  ) {
    if ("include" in rule) return { include: unSigil(rule.include, "#") }
    return rule
  }

  if ("begin" in rule) {
    const { begin, end, type, embedded, rules } = rule
    return removeUndefined({
      begin: demangleRule(begin) as Rule | RuleState,
      end: demangleRule(end) as Rule,
      type: demangleType(type),
      embedded,
      rules: rules
        ? isString(rules)
          ? demangleNext(rules)
          : rules.map(rule => demangleRule(rule))
        : undefined
    })
  }

  let target!: DF.Substitute
  let match!: DF.Match
  let action!: string | DF.Action | DF.ActionObject | DF.SubState | []

  if (isArray(rule)) {
    if (isSubRule(rule)) [target, match, ...action] = rule
    else [match, ...action] = rule
  } else if (isSubRule(rule)) {
    ;({ target, match, ...action } = rule)
  } else {
    ;({ match, ...action } = rule)
  }

  return removeUndefined({ target, match, action: demangleAction(action) })
}

export function demangleAction(
  action: string | DF.Action | DF.ActionObject | DF.SubState | []
): Action {
  if (isString(action)) return { type: demangleType(action) }

  let parsed: any = {}

  if (!isArray(action)) parsed = klona(action) as Action
  // incompatible types are addressed later
  else {
    const [idx0, idx1, idx2] = klona(action)

    // check for substate
    if (isObjectLike<DF.SubState>(idx0)) {
      parsed = idx0
    } else if (isString(idx0) && isObjectLike<DF.SubState>(idx1)) {
      parsed = { type: idx0, ...idx1 }
    }
    // normal action
    else {
      // find action object - if it exists
      const last = action[action.length - 1]
      if (last && !isString(last) && !isArray(last) && !has("rules", last)) {
        parsed = klona(last) as DF.ActionObject
      }

      // check idx0 (type string or group)
      if (isString(idx0)) parsed.type = idx0
      else if (isArray(idx0)) parsed.group = idx0

      // check idx1 (next or group)
      if (isString(idx1)) parsed.next = idx1
      else if (isArray(idx1)) parsed.group = idx1

      // check idx2 (next)
      if (isString(idx2)) parsed.next = idx2

      // idx3 will always be undefined or opts
    }
  }

  // demangle group
  if (parsed.group) {
    const groups: Action[] = []
    for (const action of parsed.group as DF.Group) {
      groups.push(demangleAction(action))
    }
    parsed.group = groups
  }

  // demangle rules
  if (parsed.rules) {
    if (isString(parsed.rules)) {
      parsed.rules = demangleNext(parsed.rules)
    } else {
      const rules: (DF.Directive | Rule | RuleState)[] = []
      for (const rule of parsed.rules) {
        rules.push(demangleRule(rule))
      }
      parsed.rules = rules
    }
  }

  if (parsed.parser) {
    const { open, close } = demangleParser(parsed.parser)
    delete parsed.parser
    parsed.open = open
    parsed.close = close
  }

  // unmangle shorthands and sigil characters
  const { type, next, switchTo } = parsed as Action
  if (type) parsed.type = unSigil(type, "t.")
  if (next) parsed.next = unSigil(next, "#")
  if (switchTo) parsed.switchTo = unSigil(switchTo, "#")

  return removeUndefined(parsed)
}

function demangleType(str?: string) {
  if (!str) return undefined
  return unSigil(str, "t.")
}

function demangleNext(str?: string) {
  if (!str) return undefined
  return unSigil(str, "#")
}

function demangleParser(parser: DF.ParserTarget | DF.ParserTarget[]) {
  const open: ParserAction = []
  const close: ParserAction = []
  ;(isArray(parser) ? parser : [parser]).forEach(parse => {
    const opening = parse[2] !== "/"
    const inclusive = +hasSigil(parse, ">>")
    const type = unSigil<string>(parse, ["t.", ">>", "<<", "/"])
    ;(opening ? open : close).push([type, inclusive])
  })
  return { open, close }
}

function isSubRule(rule: DF.Directive | DF.Rule | DF.SubRule): rule is DF.SubRule {
  if (!rule) return false
  if (isArray(rule)) return hasSigil(rule[0], ["$", "::"])
  return "target" in rule
}
