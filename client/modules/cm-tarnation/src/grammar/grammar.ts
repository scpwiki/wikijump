import { NodeProp, NodePropSource } from "@lezer/common"
import { styleTags, tags } from "@wikijump/codemirror/cm"
import {
  createID,
  escapeRegExp,
  hasSigil,
  removeUndefined,
  unSigil
} from "@wikijump/util"
import { isArray, isRegExp, isString } from "is-what"
import { klona } from "klona"
import { Memoize } from "typescript-memoize"
import { Action } from "./action"
import type * as DF from "./definition"
import type * as DM from "./demangler"
import { demangleGrammar } from "./demangler"
import { Matched } from "./matched"
import { Rule } from "./rule"
import type { SubState } from "./substate"

export interface GrammarMatchState {
  state: string
  context: DF.Context
  last?: string[]
}

export interface GrammarContext extends GrammarMatchState {
  target?: DF.SubRuleTarget
  substate?: SubState
}

export function getMatchState(cx: GrammarContext): GrammarMatchState {
  const { context, state, last } = cx
  return { context, state, last }
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

const SUB_REGEX = /\$(?:S|#|\d+)|::[^\s!]+/g
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
    { context = {}, state = "", last: [total = "", ...captures] = [] }: GrammarMatchState,
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
    if (name.indexOf("_::_") !== -1) {
      console.warn(`Grammar: Invalid bracket name '${name}'! Ignoring...`)
      return
    }

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

  // I'm very sorry about this code
  // legitimately was extremely frustrating and this is obviously sphagetti
  // Fixing this properly will involve writing a grammar definition format that isn't
  // so flat -- as in it handles recursive stuff naturally,
  // rather than with syntax sugar

  private processRuleState(
    state: string,
    rule: DM.RuleState & { begin: DM.Rule; end: DM.Rule }
  ) {
    const { embedded, type, rules } = rule
    const begin: DM.Rule = rule.begin
    const end: DM.Rule = rule.end

    begin.action ??= {}
    end.action ??= {}

    if (type) {
      begin.action.open ??= []
      end.action.close ??= []
      begin.action.open.unshift([type, 1])
      end.action.close.push([type, 1])
    }

    const finalize = (next?: string) => {
      const ids = new Set<number>()
      const endState = next ? createID("rule-state") : state

      if (embedded) {
        // don't start embedded if our rule is going off somewhere else
        if (!begin.action!.switchTo && !begin.action!.next) {
          begin.action!.embedded = embedded.slice(0, embedded.length - 1)
        }
        end.action!.embedded = "@pop"
      }

      if (embedded || rules || next) {
        if (next) {
          begin.action!.switchTo = endState
          end.action!.switchTo = next
        } else {
          if (begin.action?.switchTo) {
            begin.action.next = begin.action.switchTo
            delete begin.action.switchTo
          } else {
            begin.action!.next = state
          }
          end.action!.next = "@pop"
        }
      } else if (!embedded && !rules && begin.action!.switchTo) {
        begin.action!.next = begin.action!.switchTo
        delete begin.action!.switchTo
      }

      if (rules) {
        const stateRules = [end, ...(isString(rules) ? [{ include: rules }] : rules)]
        this.states.set(endState, this.addRules(stateRules as any))
      } else if (embedded || next) {
        this.states.set(endState, new Set([this.addRule(end).id]))
      }

      if (next) {
        this.states.set(state, new Set([this.addRule(begin).id]))
      } else {
        ids.add(this.addRule(begin).id)
        if (!embedded && !rules) ids.add(this.addRule(end).id)
      }

      return ids
    }

    return { begin, end, finalize }
  }

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
      try {
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
        // rule state (incoming sphagetti, see above comments)
        else if ("begin" in rule) {
          const root = klona(rule)
          const stateRoot = createID("rule-state")

          const finalizeList: AnyFunction[] = []

          // I've left this as an inline function for now
          // The reason for its existence before was that `root.begin` and `root.end`
          // were previously allowed to be nested deeply, but now only `root.begin` is
          // This doesn't need to be a function but that `root.end` capability may get
          // added so that's why I've left it

          const resolveNesting = (child: DM.RuleState) => {
            const stack: [string, DM.RuleState][] = []

            // assemble list of nested rulestates
            let current: DM.Rule | DM.RuleState = child
            while ("begin" in current) {
              stack.push([createID("rule-state"), current])
              current = current.begin
            }

            // reverse list so that deepest node comes first
            stack.reverse()

            // set deepest end rule to start the embedded range
            if (root.embedded) {
              const embedded = root.embedded
              const state = stack[0][1]
              if ("begin" in state) {
                state.end.action ??= {}
                state.end.action.embedded = embedded.slice(0, embedded.length - 1)
              }
            }

            // work deepest to shallowest, and eventually propagate
            // the deepest node to the root, which serves as the entrypoint to the chain
            stack.reduce(([prev, last], [next, state]) => {
              const { begin, end, finalize } = this.processRuleState(prev, last as any)
              finalizeList.push(() => finalize(next))
              if ("begin" in state) {
                state.begin = begin
                state.end = end
              }
              return [next, state]
            })

            // workaround for rulestates that aren't supposed to change the stack state
            if (!root.embedded && !root.rules) {
              finalizeList.push(() => {
                const state = stack[0][1]
                if ("begin" in state) {
                  state.end.action ??= {}
                  state.end.action.next = "@pop"
                  delete state.end.action.switchTo
                }
              })
            }
          }

          if ("begin" in root.begin) {
            resolveNesting(root.begin)
            const state = createID("rule-state")
            const { begin, finalize } = this.processRuleState(state, root.begin)
            finalizeList.push(() => finalize(stateRoot))
            root.begin = begin
          }

          // finalize everything
          const { finalize } = this.processRuleState(stateRoot, root)
          finalizeList.forEach(fn => fn())
          finalize().forEach(id => ids.add(id))
        }
        // normal rule
        else {
          ids.add(this.addRule(rule).id)
        }
      } catch (err) {
        console.warn("Grammar: Failed to add rule. Ignoring...")
        console.warn(err)
        console.info(rule)
      }
    }
    return ids
  }

  // STATES

  addState(name: string): Set<number> {
    if (this.states.has(name)) return this.states.get(name)!
    const states = this.grammar.states
    const state = states[name]
    if (!state) {
      console.warn(`Grammar: Undefined state (${name}) added in grammar! Ignoring...`)
      return new Set()
    }

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

  @Memoize()
  private getRulesForState(state: SubState | string | Set<number>) {
    const ids = state instanceof Set ? state : this.states.get(state)
    if (!ids) return null
    const rules = new Set<Rule>()
    for (const id of ids) {
      rules.add(this.rules.get(id)!)
    }
    return rules
  }

  match(cx: GrammarContext, str: string, pos: number, offset = 0): Matched | null {
    const rules = this.getRulesForState(cx.substate ?? cx.state)
    if (!rules) {
      console.warn(`Grammar: Undefined state! (${cx.substate ?? cx.state})`)
      return null
    }

    for (const rule of rules) {
      const matches = rule.exec(cx, str, pos)
      if (!matches) continue
      if (offset !== pos) matches.offset = offset
      if (rule.log) console.log(Grammar.sub(cx, rule.log))
      return matches
    }

    if (!cx.substate?.strict) {
      for (const rule of this.getRulesForState(this.global)!) {
        const matches = rule.exec(cx, str, pos)
        if (!matches) continue
        if (offset !== pos) matches.offset = offset
        if (rule.log) console.log(Grammar.sub(cx, rule.log))
        return matches
      }
    }

    if (this.fallback) {
      return new Matched(str[pos], this.fallback, offset, getMatchState(cx), cx.context)
    }

    return null
  }
}
