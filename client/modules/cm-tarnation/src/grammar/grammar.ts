import { styleTags, tags } from "@codemirror/highlight"
import { isArray, isRegExp, isString } from "is-what"
import { klona } from "klona"
import { NodeProp, NodePropSource } from "lezer-tree"
import { createID, escapeRegExp, hasSigil, removeUndefined, unSigil } from "wj-util"
import { Action } from "./action"
import type * as DF from "./definition"
import type * as DM from "./demangler"
import { demangleGrammar } from "./demangler"
import { Matched } from "./matched"
import { Rule } from "./rule"
import type { SubState } from "./substate"

/** Stores information about the current state of the {@link Grammar} match. */
export interface GrammarContext {
  state: string
  context: DF.Context
  last?: string[]
  target?: DF.SubRuleTarget
  substate?: SubState
}

export function createContext(state: string, context: DF.Context = {}): GrammarContext {
  return { state, context }
}

/** Represents an individual token emitted by a {@link Grammar}. */
export interface GrammarToken {
  type: string
  from: number
  to: number
  empty: boolean

  open?: ParserAction
  close?: ParserAction
  next?: string
  switchTo?: string
  embedded?: string
  context?: DF.Context
}

/** Directs the parser to nest tokens. */
export type ParserAction = [type: string, inclusive: number][]

const SUB_REGEX = /\$(?:S|#|\d+)|::\S+/g
const BRACKET_REGEX = /@BR(\/[OC])?(?::(.+))?/

export class Grammar {
  private declare grammar: DM.Grammar

  declare ignoreCase: boolean
  declare variables: Record<string, DF.Variable>
  declare start: string
  declare fallback: Action

  types = new Set<string>()
  props: NodePropSource[] = []

  private declare global: Set<number>

  private brackets = new Map<string, Action>()
  private rules = new Map<number, Rule>()
  private states = new Map<string | SubState, Set<number>>()

  private includeMap = new Map<Set<number>, Set<string>>()

  constructor(def: DF.Grammar) {
    const grammar = demangleGrammar(def)
    const { ignoreCase = false, variables = {}, start = "root" } = grammar

    this.grammar = grammar
    this.ignoreCase = ignoreCase
    this.variables = variables
    this.start = start

    this.init(grammar)
  }

  private init(def: DM.Grammar) {
    const { fallback, brackets = [], global = [], states } = def

    if (fallback) this.fallback = new Action(this, fallback)

    this.global = this.addRules(global)

    for (const bracket of brackets) this.addBracket(bracket)
    for (const name in states) this.addState(name)

    this.includeMap.forEach((includes, state) => {
      for (const include of includes) {
        const ids = this.addState(include)
        ids.forEach(id => state.add(id))
      }
    })
  }

  // PUBLIC UTILITY METHODS

  static sub(
    { context = {}, state = "", last: [total = "", ...captures] = [] }: GrammarContext,
    str: string
  ) {
    return str.replace(SUB_REGEX, (sub: string) => {
      if (sub === "$#") return total
      if (sub === "$S") return state
      if (sub.startsWith("::")) return context[sub.slice(2)] ?? ""
      return [total, ...captures][parseInt(sub.slice(1))] ?? ""
    })
  }

  static variableForRegex(variable?: DF.Variable | null) {
    if (isString(variable)) return escapeRegExp(variable)
    if (isRegExp(variable)) return variable.source
    if (isArray(variable)) return variable.map(str => escapeRegExp(str)).join("|")
    return null
  }

  static variableForString(variable?: DF.Variable | null) {
    if (isString(variable)) return variable
    if (isRegExp(variable)) return variable.source
    return null
  }

  /** Resolves and replaces the `@` variables in a `RegExp` or `string`. */
  static expand(vars: Record<string, DF.Variable>, input: string | RegExp) {
    const regex = isRegExp(input)
    let count = 0
    let str = regex ? (input as RegExp).source : (input as string)
    while (/@\w/.test(str) && count < 5) {
      count++
      str = str.replace(/@(\w+)/g, (_, ident: string) => {
        const variable = regex
          ? this.variableForRegex(vars[ident])
          : this.variableForString(vars[ident])
        if (!variable) return ""
        return regex ? `(?:${variable})` : variable
      })
    }
    return str
  }

  // BRACKETS

  // str_::_mode_::_hint
  private static makeBracketString(str: string, mode: boolean | null, hint: string) {
    return `${str}${mode !== null ? `_::_${mode ? "open" : "close"}` : ""}${
      hint ? `_::_${hint}` : ""
    }`
  }

  findBracket(str: string, type: string) {
    if (!BRACKET_REGEX.test(type)) return null
    const [, modeStr, hint = ""] = BRACKET_REGEX.exec(type)!
    const mode = modeStr === "/O" ? true : modeStr === "/C" ? false : null
    const bracket = Grammar.makeBracketString(str.trim(), mode, hint)
    return this.brackets.get(bracket) ?? null
  }

  private addBracket({ name, pair, hint = "", tag, parented }: DF.Bracket) {
    if (name.indexOf("_::_") !== -1) throw new Error("Invalid bracket name!")

    const tagged = hasSigil(name, "t.")
    const openType = tagged ? unSigil(name, "t.") : (`${name}Open` as DF.Type)
    const closeType = tagged ? unSigil(name, "t.") : (`${name}Close` as DF.Type)

    if (pair) {
      const [open, close] = isString(pair) ? [pair, pair] : pair

      const openAction = new Action(this, { type: openType })
      const closeAction = new Action(this, { type: closeType })

      if (open !== close) {
        this.brackets.set(Grammar.makeBracketString(open, null, hint), openAction)
        this.brackets.set(Grammar.makeBracketString(close, null, hint), closeAction)
      }

      this.brackets.set(Grammar.makeBracketString(open, true, hint), openAction)
      this.brackets.set(Grammar.makeBracketString(close, false, hint), closeAction)
    } else {
      this.types.add(unSigil(openType, "t."))
      this.types.add(unSigil(closeType, "t."))
    }

    if (!tagged) {
      if (parented === undefined && !pair) parented = true
      const parent = parented ? `${name}/` : ""
      this.props.push(
        NodeProp.openedBy.add({ [closeType]: [openType] }),
        NodeProp.closedBy.add({ [openType]: [closeType] })
      )
      if (tag) {
        this.props.push(
          styleTags({
            // @ts-ignore
            [`${parent}${openType} ${parent}${closeType}`]: tags[tag.slice(2)]!
          })
        )
      }
    }
  }

  // RULES

  private addRule(def: DM.Rule) {
    const id = this.rules.size
    // takes the rule slot to avoid rules being overwritten
    this.rules.set(id, null as any)
    const rule = new Rule(this, id, def)
    this.rules.set(id, rule)
    return rule
  }

  private addRules(rules: (DF.Directive | DM.Rule | DM.RuleState)[]): Set<number> {
    const ids = new Set<number>()
    const includes = new Set<string>()
    this.includeMap.set(ids, includes)

    for (const rule of rules) {
      // include directive
      if ("include" in rule) {
        this.addState(rule.include)
        includes.add(rule.include)
      }
      // props directive
      else if ("props" in rule) {
        this.props.push(...rule.props)
      }
      // style directive
      else if ("style" in rule) {
        this.props.push(styleTags(removeUndefined(rule.style)))
      }
      // brackets directive
      else if ("brackets" in rule) {
        for (const bracket of rule.brackets) this.addBracket(bracket)
      }
      // variables directive
      else if ("variables" in rule) {
        Object.assign(this.variables, rule.variables)
      }
      // rule state
      else if ("begin" in rule) {
        try {
          const state = createID("rule-state")

          const { embedded, type, rules } = rule

          const begin = klona(rule.begin)
          const end = klona(rule.end)

          begin.action ??= {}
          end.action ??= {}

          if (type) {
            begin.action.open ??= []
            end.action.close ??= []
            begin.action.open.unshift([type, 1])
            end.action.close.push([type, 1])
          }

          if (embedded || rules) {
            begin.action.next = state
            end.action.next = "@pop"
          }

          if (embedded) {
            begin.action.embedded = embedded.slice(0, embedded.length - 1)
            end.action.embedded = "@pop"
          }

          if (embedded) {
            this.states.set(state, new Set([this.addRule(end).id]))
          } else if (rules) {
            const stateRules = [end, ...(isString(rules) ? [{ include: rules }] : rules)]
            this.states.set(state, this.addRules(stateRules as any))
          }

          ids.add(this.addRule(begin).id)
          if (!embedded && !rules) ids.add(this.addRule(end).id)
        } catch (err) {
          console.warn("Grammar: Failed to add rule state. Ignoring...")
          console.info(rule)
        }
      }
      // normal rule
      else {
        try {
          ids.add(this.addRule(rule).id)
        } catch (err) {
          console.warn("Grammar: Failed to add rule. Ignoring...")
          console.info(rule)
        }
      }
    }
    return ids
  }

  // STATES

  addState(name: string): Set<number> {
    if (this.states.has(name)) return this.states.get(name)!
    const states = this.grammar.states
    const state = states[name]
    if (!state) throw new Error("Undefined state specified in grammar!")

    this.states.set(name, new Set()) // prevents cyclic
    const ids = this.addRules(state)
    this.states.set(name, ids)

    return ids
  }

  addSubstate(
    substate: SubState,
    rules: (DF.Directive | DM.Rule | DM.RuleState)[] | Set<number>
  ) {
    if (this.states.has(substate)) return this.states.get(substate)!

    this.states.set(substate, new Set()) // prevents cyclic
    const ids = rules instanceof Set ? rules : this.addRules(rules)
    this.states.set(substate, ids)

    return ids
  }

  // MATCH

  match(cx: GrammarContext, str: string, pos: number, offset = 0): Matched | null {
    const ids = this.states.get(cx.substate ?? cx.state)
    if (!ids) throw new Error("Undefined state specified in grammar!")

    for (const id of ids) {
      const rule = this.rules.get(id)!
      const matches = rule.exec(cx, str, pos)
      if (!matches) continue
      if (offset !== pos) matches.offset = offset
      if (rule.log) console.log(rule.log)
      return matches
    }

    if (!cx.substate?.strict) {
      for (const id of this.global) {
        const rule = this.rules.get(id)!
        const matches = rule.exec(cx, str, pos)
        if (!matches) continue
        if (offset !== pos) matches.offset = offset
        if (rule.log) console.log(rule.log)
        return matches
      }
    }

    if (this.fallback) return new Matched(str[pos], this.fallback, offset, cx.context)

    return null
  }
}
