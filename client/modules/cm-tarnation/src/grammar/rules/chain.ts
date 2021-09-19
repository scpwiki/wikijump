import type * as DF from "../definition"
import { Matched } from "../matched"
import { RegExpMatcher } from "../matchers/regexp"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

export class Chain extends Rule {
  private declare chain: ChainRule[]
  private declare skip?: RegExpMatcher

  constructor(repo: Repository, rule: DF.Chain) {
    super(repo, rule)
    if (rule.skip) {
      this.skip = new RegExpMatcher(rule.skip, repo.ignoreCase, repo.variables)
    }
    this.chain = rule.chain.map(item => parseChainRule(repo, item))
  }

  exec(state: GrammarState, str: string, pos: number) {
    const ctx = new ChainContext(state, this.chain, str, pos, this.skip)

    while (!ctx.done) step(ctx)

    const finished = ctx.finish()

    if (!finished) return null

    return new Matched(state, this.node, ctx.total, pos, finished)
  }
}

function step(ctx: ChainContext) {
  ctx.skip()
  step: switch (ctx.current[1]) {
    case Quantifier.ONE: {
      const result = ctx.current[0].match(ctx.state, ctx.str, ctx.pos)
      if (result) ctx.addAndAdvance(result)
      else ctx.fail()
      break
    }

    case Quantifier.OPTIONAL: {
      const result = ctx.current[0].match(ctx.state, ctx.str, ctx.pos)
      if (result) ctx.add(result)
      ctx.advance()
      break
    }

    case Quantifier.ZERO_OR_MORE: {
      take(ctx, ctx.current[0])
      ctx.advance()
      break
    }

    case Quantifier.ONE_OR_MORE: {
      const advanced = take(ctx, ctx.current[0])
      if (advanced) ctx.advance()
      else ctx.fail()
      break
    }

    case Quantifier.ALTERNATIVES: {
      const rules = ctx.current[0]

      let advanced = false
      let couldFail = false

      for (let i = 0; i < rules.length; i++) {
        const type = rules[i][1]
        if (type === Quantifier.ONE || type === Quantifier.ONE_OR_MORE) {
          couldFail = true
        }

        switch (type) {
          case Quantifier.ONE:
          case Quantifier.OPTIONAL: {
            const result = rules[i][0].match(ctx.state, ctx.str, ctx.pos)
            if (result) {
              ctx.add(result)
              advanced = true
            }
            break
          }
          case Quantifier.ZERO_OR_MORE:
          case Quantifier.ONE_OR_MORE: {
            advanced = take(ctx, rules[i][0])
            break
          }
        }

        // leave for loop if we advanced
        if (advanced) break
      }

      if (!advanced && couldFail) ctx.fail()
      else ctx.advance()
      break
    }

    case Quantifier.REPEATING_ZERO_OR_MORE: {
      const rules = ctx.current[0]
      for (let i = 0; i < rules.length; i++) {
        const result = rules[i].match(ctx.state, ctx.str, ctx.pos)
        if (result) {
          ctx.add(result)
          break step
        }
      }
      ctx.advance()
      break
    }

    case Quantifier.REPEATING_ONE_OR_MORE: {
      if (ctx.advanced === null) ctx.advanced = false
      const rules = ctx.current[0]
      for (let i = 0; i < rules.length; i++) {
        const result = rules[i].match(ctx.state, ctx.str, ctx.pos)
        if (result) {
          ctx.add(result)
          ctx.advanced = true
          break step
        }
      }
      if (!ctx.advanced) ctx.fail()
      else ctx.advance()
      ctx.advanced = null
      break
    }
  }
}

function take(ctx: ChainContext, rule: Rule) {
  let advanced = false
  let result
  while ((result = rule.match(ctx.state, ctx.str, ctx.pos))) {
    ctx.add(result)
    ctx.skip()
    advanced = true
  }
  return advanced
}

enum Quantifier {
  ONE,
  OPTIONAL,
  ZERO_OR_MORE,
  ONE_OR_MORE,
  ALTERNATIVES,
  REPEATING_ZERO_OR_MORE,
  REPEATING_ONE_OR_MORE
}

// prettier-ignore
type ChainRuleSimple = [Rule,
  | Quantifier.ONE
  | Quantifier.OPTIONAL
  | Quantifier.ZERO_OR_MORE
  | Quantifier.ONE_OR_MORE
]

// prettier-ignore
type ChainRule =
  | ChainRuleSimple
  | [ChainRuleSimple[], Quantifier.ALTERNATIVES]
  | [Rule[],
      | Quantifier.REPEATING_ZERO_OR_MORE
      | Quantifier.REPEATING_ONE_OR_MORE
    ]

class ChainContext {
  declare state: GrammarState
  declare rules: ChainRule[]
  declare str: string
  declare pos: number
  declare total: string
  declare results: Matched[]
  declare index: number
  declare failed: boolean
  declare advanced: boolean | null
  declare skipMatcher?: RegExpMatcher

  constructor(
    state: GrammarState,
    rules: ChainRule[],
    str: string,
    pos: number,
    skip?: RegExpMatcher
  ) {
    this.state = state
    this.rules = rules
    this.str = str
    this.pos = pos
    this.total = ""
    this.results = []
    this.index = 0
    this.failed = false
    this.advanced = null
    if (skip) this.skipMatcher = skip
  }

  get done() {
    return this.index >= this.rules.length
  }

  get current() {
    if (this.done) throw new Error("Cannot get current rule when done")
    return this.rules[this.index]
  }

  add(...results: Matched[]) {
    for (const result of results) {
      this.results.push(result)
      this.total += result.total
      this.pos += result.length
    }
  }

  fail() {
    this.failed = true
    this.index = this.rules.length
  }

  advance() {
    this.index++
  }

  addAndAdvance(result: Matched) {
    this.add(result)
    this.advance()
  }

  finish() {
    if (this.failed) return null
    return this.results
  }

  skip() {
    if (!this.skipMatcher) return
    let result
    while ((result = this.skipMatcher.match(this.str, this.pos))) {
      this.pos += result.length
      this.total += result.total
    }
  }
}

function parseChainRule(repo: Repository, str: string): ChainRule {
  const repeatAlternatives = /\|[*+]/.test(str)
  const normalAlternatives = /\|(?![*+])/.test(str)

  if (repeatAlternatives && normalAlternatives) {
    throw new Error("Cannot mix |* (or |+) and |")
  }

  if (!repeatAlternatives && !normalAlternatives) {
    let type = Quantifier.ONE
    // prettier-ignore
    switch (str[str.length - 1]) {
      case "?": type = Quantifier.OPTIONAL; break
      case "*": type = Quantifier.ZERO_OR_MORE; break
      case "+": type = Quantifier.ONE_OR_MORE; break
    }

    if (type !== Quantifier.ONE) {
      str = str.slice(0, str.length - 1)
    }

    const rule = repo.get(str)
    if (!(rule instanceof Rule)) throw new Error(`Rule "${str}" not found`)

    return [rule, type]
  }
  // normal alternatives
  else if (normalAlternatives) {
    const rules = str.split(/\s*\|\s*/).map(item => parseChainRule(repo, item))
    return [rules as ChainRuleSimple[], Quantifier.ALTERNATIVES]
  }
  // repeating alternatives
  else if (repeatAlternatives) {
    const zeroOrMore = /\|\*/.test(str)
    const oneOrMore = /\|\+/.test(str)

    if (zeroOrMore && oneOrMore) {
      throw new Error("Cannot have repeating alternatives with both * and +")
    }

    const rules = str.split(/\s*\|[*+]?\s*/).map(item => repo.get(item))

    if (rules.some(rule => !(rule instanceof Rule))) {
      throw new Error(`Rule "${str}" not found`)
    }

    return [
      rules as Rule[],
      zeroOrMore ? Quantifier.REPEATING_ZERO_OR_MORE : Quantifier.REPEATING_ONE_OR_MORE
    ]
  }

  throw new Error("Unreachable")
}
