import { styleTags, tags } from "@codemirror/highlight"
import { isArray, isFunction, isRegExp, isString } from "is-what"
import { klona } from "klona"
import { NodeProp, NodePropSource } from "lezer-tree"
import {
  createID,
  escapeRegExp,
  has,
  hasSigil,
  pointsMatch,
  removeUndefined,
  toPoints,
  unSigil
} from "wj-util"
import type * as DF from "./definition"
import type * as DM from "./demangler"
import { demangleGrammar } from "./demangler"

/** Stores information about the current state of the {@link Grammar} match. */
export interface GrammarContext {
  state: string
  context: DF.Context
  last?: string[]
  target?: DF.SubRuleTarget
  substate?: SubState
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
type ParserAction = [type: string, inclusive: number][]

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
    const bracket = Grammar.makeBracketString(str, mode, hint)
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

export class Rule {
  declare log?: string

  private declare target?: DF.SubRuleTarget
  private declare matcher: Matcher | "@DEFAULT"
  private declare action: Action

  constructor(grammar: Grammar, public id: number, rule: DM.Rule) {
    const { target, match, action } = rule

    if (!match) throw new Error("Rule must have a Match!")

    if (target) this.target = target

    if (match === "@DEFAULT") this.matcher = "@DEFAULT"
    else this.matcher = new Matcher(grammar, match)

    this.action = new Action(grammar, action)
    if (this.action.log) this.log = this.action.log
  }

  exec(cx: GrammarContext, str: string, pos: number) {
    // always match entire str past pos
    if (this.matcher === "@DEFAULT") {
      return this.action.exec(cx, [str.slice(pos)], pos)
    }

    const { target, last } = cx

    cx.target = this.target
    const found = this.matcher.exec(cx, str, pos)
    cx.target = target
    if (!found) return null

    if (this.target) {
      if (!cx.last) throw new Error("Rule target specified, but no last match!")
      return this.action.exec(cx, cx.last, pos)
    } else {
      if (!cx.last || !cx.substate) cx.last = found
      const result = this.action.exec(cx, found, pos)
      cx.last = last
      return result
    }
  }
}

const enum MatcherType {
  RegExp,
  Points,
  Substitute,
  Function
}

// prettier-ignore
type MatcherElement =
  | { matcher: DF.MatchFunction; type: MatcherType.Function }
  | { matcher: RegExp;           type: MatcherType.RegExp }
  | { matcher: DF.Substitute;    type: MatcherType.Substitute }
  | { matcher: number[];         type: MatcherType.Points; source: string }

export class Matcher {
  private declare elements?: MatcherElement[]

  constructor(grammar: Grammar, matchers: DF.Match) {
    if (!isArray(matchers)) matchers = [matchers]

    const compiled = matchers.map(matcher => {
      if (!matcher) {
        throw new Error("Null matcher given! A helper function likely errored.")
      } else if (isFunction(matcher)) return matcher
      else if (hasSigil<DF.Substitute>(matcher, ["$", "::"])) return matcher
      else return Matcher.compile(grammar, matcher!)
    })

    this.elements = compiled.map(matcher => {
      let type: MatcherType

      if (isFunction(matcher)) type = MatcherType.Function
      else if (isRegExp(matcher)) type = MatcherType.RegExp
      else if (hasSigil(matcher, ["$", "::"])) type = MatcherType.Substitute
      else type = MatcherType.Points

      if (type === MatcherType.Points) {
        const source = String.fromCodePoint(...(matcher as number[]))
        return { matcher, type, source } as MatcherElement
      }

      return { matcher, type } as MatcherElement
    })
  }

  private static compile(
    { variables, ignoreCase }: Grammar,
    matcher: RegExp | string
  ): RegExp | DF.MatchFunction | number[] {
    if (isRegExp(matcher)) {
      const str = Grammar.expand(variables, matcher)
      // prettier-ignore
      // eslint-disable-next-line
      const flags = "ym" +
        (matcher.dotAll ? "s" : "") +
        (matcher.unicode ? "u" : "") +
        (ignoreCase || matcher.ignoreCase ? "i" : "")
      return new RegExp(str, flags)
    } else {
      // entire string is a variable
      if (/^@\w+$/.test(matcher)) {
        const variable = variables[matcher.slice(1)]
        if (isFunction(variable)) return variable
        else if (isRegExp(variable)) {
          return this.compile({ variables, ignoreCase } as Grammar, variable)
        }
      }
      return toPoints(Grammar.expand(variables, matcher))
    }
  }

  exec(cx: GrammarContext, str: string, pos: number): string[] | null {
    if (!this.elements) return null

    if (cx.target && cx.last) {
      const { state, last, target, context } = cx
      if (hasSigil(target, "$")) {
        if (target === "$#") str
        else if (target === "$S") str = state
        else str = last[parseInt(target.slice(1))] ?? ""
      } else {
        const prop = context[str.slice(2)]
        if (!prop) return null
        str = prop
      }
    }

    const found: string[] = [""]
    const start = pos

    // for loop is faster and doesn't invoke iterator methods needlessly
    for (let i = 0; i < this.elements.length; i++) {
      let total: string | null = null
      let match: string[] | null = null

      const element = this.elements[i]

      switch (element.type) {
        case MatcherType.Function: {
          match = element.matcher(cx, str, pos)
          break
        }

        case MatcherType.RegExp: {
          element.matcher.lastIndex = pos
          const result = element.matcher.exec(str)
          if (result) [total, ...match] = result
          break
        }

        case MatcherType.Substitute: {
          const sub = Grammar.sub(cx, element.matcher)
          if (!sub) break
          if (pointsMatch(toPoints(sub), str, pos)) match = [sub]
          break
        }

        case MatcherType.Points: {
          if (pointsMatch(element.matcher, str, pos)) {
            match = [element.source]
          }
          break
        }

        default: {
          return null
        }
      }

      if (!total && !match) return null

      // skip further checking if we don't need to
      if (this.elements.length === 1) {
        if (total) return [total, ...(match ?? [])]
        else if (match) return [match.join(), ...match]
      }

      if (total) found[0] += total

      if (match?.length) {
        if (!total) found[0] = found[0].concat(...match)
        found.push(...match)
      } else if (total) {
        found.push(total)
      }

      if (found[0] && found.length === 1) found[1] = found[0]

      pos = start + found[0].length
    }

    if (!found.length || (found.length === 1 && !found[0])) return null

    return found
  }
}

const enum ActionMode {
  Normal,
  SubState,
  Group,
  Bracket,
  Rematch
}

export class Action {
  declare type: string
  declare group?: Action[]
  declare next?: string
  declare log?: string
  declare switchTo?: string
  declare open?: ParserAction
  declare close?: ParserAction
  declare embedded?: string
  declare context?: DF.Context

  private declare substate?: SubState

  declare mode: ActionMode

  constructor(private grammar: Grammar, action?: DM.Action) {
    this.type = ""

    if (action) {
      const { type, next, open, close, log, switchTo, embedded, context } = action
      Object.assign(
        this,
        removeUndefined({ type, next, open, close, log, switchTo, embedded, context })
      )
      if (has("rules", action)) this.substate = new SubState(grammar, action)
      else if (action.group) {
        this.group = action.group.map(act => new Action(grammar, act))
      }
    }

    if (this.substate) this.mode = ActionMode.SubState
    else if (hasSigil(this.type, "@RE")) this.mode = ActionMode.Rematch
    else if (hasSigil(this.type, "@BR")) this.mode = ActionMode.Bracket
    else if (this.group) this.mode = ActionMode.Group
    else this.mode = ActionMode.Normal

    // add types
    if (this.type && !hasSigil(this.type, "@")) grammar.types.add(this.type)
    if (this.open) for (const [type] of this.open) grammar.types.add(type)
    if (this.close) for (const [type] of this.close) grammar.types.add(type)
  }

  private setContext(cx: GrammarContext) {
    if (!this.context) return cx.context
    const added: DF.Context = {}
    for (const key in this.context) {
      const prop = this.context[key]
      if (prop) added[key] = Grammar.sub(cx, prop)
    }
    return (cx.context = { ...cx.context, ...added })
  }

  exec(cx: GrammarContext, found: string[], pos: number): Matched | null {
    const matched = new Matched(found[0], this, pos)

    switch (this.mode) {
      case ActionMode.Normal: {
        matched.context = this.setContext(cx)
        return matched
      }

      case ActionMode.Bracket: {
        matched.context = this.setContext(cx)
        const action = this.grammar.findBracket(found[0], this.type)
        if (action) return Matched.extend(matched, [new Matched(found[0], action, pos)])
        return null
      }

      case ActionMode.Rematch: {
        return new Matched("", this, pos, this.setContext(cx))
      }

      case ActionMode.SubState: {
        const sub = this.substate!.exec(cx, found[0], pos)
        if (!sub) return null
        matched.context = this.setContext(cx)
        return Matched.extend(matched, sub)
      }

      case ActionMode.Group: {
        const [, ...captures] = found
        let offset = pos
        const group: (Matched | null)[] = []
        for (let i = 0; i < captures.length; i++) {
          const capture = captures[i]
          const action = this.group![i]
          if (!action) throw new Error("Disjointed match group count!")
          if (!capture) continue
          group.push(action.exec(cx, [capture], offset))
          offset += capture.length
        }
        matched.context = this.setContext(cx)
        return Matched.extend(matched, group)
      }

      default: {
        return null
      }
    }
  }
}

export class SubState {
  declare repeat: boolean
  declare optional: boolean
  declare all: boolean
  declare strict: boolean

  constructor(
    public grammar: Grammar,
    { strict = true, repeat, optional, all, rules }: DM.Action
  ) {
    if (!rules) throw new Error("Substate must have rules!")

    this.repeat = strict ? false : repeat ?? true
    this.optional = strict ? false : optional ?? true
    this.all = strict ? true : all ?? false
    this.strict = strict

    if (isString(rules)) grammar.addSubstate(this, grammar.addState(rules))
    else grammar.addSubstate(this, rules)
  }

  exec(cx: GrammarContext, str: string, pos: number): Matched[] | null {
    const lastSubstate = cx.substate
    const matches: Matched[] = []

    cx.substate = this

    let offset = 0
    let match: Matched | null = null
    // prettier-ignore
    while (offset < str.length &&
      ((match = this.grammar.match(cx, str, offset)) || (!this.all && this.repeat))
    ) {
      if (!match) {
        offset += 1
        continue
      }
      match.offset = pos + offset
      matches.push(match)
      offset += match.length
      if (!this.repeat) break
    }

    cx.substate = lastSubstate

    if (!this.optional && !matches.length) return null
    if (this.all && offset < str.length) return null
    return matches
  }
}

// MISC.

export function createContext(state: string, context: DF.Context = {}): GrammarContext {
  return { state, context }
}

export function createToken({ from, to, action, context }: Matched): GrammarToken {
  const { type, open, close, next, switchTo, embedded } = action
  const empty = !(type || open || close || next || switchTo || embedded || context)
  return { type, from, to, empty, open, close, next, switchTo, embedded, context }
}

/**
 * Wraps a list of {@link GrammarToken} with the token data of a {@link Match}.
 *
 * Effectively, this mutates the list of tokens as if the given
 * {@link Match} "was" the list of tokens. If the token data of the match
 * were to cause the tokenizer to manipulate the stack, it will make the
 * token list given do the same.
 */
export function wrapTokens(
  tokens: GrammarToken[],
  { context, action: { type, mode, next, switchTo, open, close, embedded } }: Matched
) {
  const first = tokens[0]
  const last = tokens[tokens.length - 1]

  if (context) last.context = { ...last.context, ...context }

  if (next || switchTo) {
    tokens.unshift(
      createToken({
        from: last.to,
        to: last.to,
        action: { type: "", next, switchTo }
      } as Matched)
    )
  }

  if (embedded && !embedded.endsWith("!")) {
    if (embedded === "@pop") first.embedded = embedded
    else last.embedded = embedded
  }

  if (type || open || close) {
    // add arrays if missing
    open ??= []
    close ??= []
    first.open ??= []
    last.close ??= []
    if (type && mode !== ActionMode.Bracket && mode !== ActionMode.Rematch) {
      first.open.unshift(...klona(open), [type, 1])
      last.close.push([type, 1], ...klona(close))
    } else {
      first.open.unshift(...klona(open))
      last.close.push(...klona(close))
    }
  }

  return tokens
}

export interface MatchedOpts {
  total: string
  action: Action
  captures?: Matched[]
  offset?: number
  context?: DF.Context
}

export class Matched {
  declare total: string
  declare action: Action
  declare captures: Set<Matched>
  declare size: number
  declare length: number
  declare from: number
  declare to: number
  declare context?: DF.Context

  constructor(
    total: string,
    action: Action,
    offset: number,
    context?: DF.Context,
    captures?: Matched[]
  ) {
    this.total = total
    this.action = action
    this.captures = new Set(captures)
    this.size = this.captures.size
    this.length = total.length
    this.from = offset
    this.to = total.length + offset
    this.context = context
  }

  set offset(offset: number) {
    for (const match of this.captures) {
      match.offset = match.from - this.from + offset
    }
    this.from = offset
    this.to = this.total.length + offset
  }

  compile(): GrammarToken[] {
    if (!this.size) return [createToken(this)]

    const tokens: GrammarToken[] = []
    for (const match of this.captures) {
      for (const token of match.compile()) {
        tokens.push(token)
      }
    }

    return wrapTokens(tokens, this)
  }

  static extend(matched: Matched | null, captures?: (Matched | null)[]) {
    if (!matched) return null
    if (!captures || captures.length === 0) return matched
    for (let i = 0; i < captures.length; i++) {
      if (!captures[i]) return null
    }
    matched.captures = new Set(captures as Matched[])
    matched.size = matched.captures.size
    return matched
  }

  *[Symbol.iterator]() {
    yield* this.captures
  }
}
