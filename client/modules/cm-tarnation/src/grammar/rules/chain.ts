import type * as DF from "../definition"
import { Matched } from "../matched"
import { RegExpMatcher } from "../matchers/regexp"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

/** A {@link Rule} subclass that uses *other* {@link Rule}s to chain together matches. */
export class Chain extends Rule {
  /** The internal list of rules and their quantifier types. */
  private declare chain: ChainRule[]

  /**
   * A {@link RegExpMatcher} pattern that, if provided, will be used to skip
   * characters which are matched. This can be used to skip whitespace in a
   * chain without making sure every rule actually handles whitespace.
   */
  private declare skip?: RegExpMatcher

  /**
   * Internal {@link ChainContext} used for keeping track of state when
   * checking if this rule matches.
   */
  private declare context: ChainContext

  /**
   * @param repo - The {@link Repository} to add this rule to.
   * @param rule - The rule definition.
   */
  constructor(repo: Repository, rule: DF.Chain) {
    super(repo, rule)
    this.chain = rule.chain.map(item => parseChainRule(repo, item))
    if (rule.skip) {
      this.skip = new RegExpMatcher(rule.skip, repo.ignoreCase, repo.variables)
    }
    this.context = new ChainContext(this.chain, this.skip)
  }

  /**
   * @param state - The current {@link GrammarState}.
   * @param str - The string to match.
   * @param pos - The position to start matching at.
   */
  exec(str: string, pos: number, state: GrammarState) {
    this.context.reset(state, str, pos)
    while (!this.context.done) step(this.context)
    const finished = this.context.finish()
    if (!finished) return null
    return new Matched(state, this.node, this.context.total, pos, finished)
  }
}

/**
 * Step function for a running chain match. A step may or may not advance
 * the chain - this is simply repeated as many times as needed.
 */
function step(ctx: ChainContext) {
  ctx.skip()
  step: switch (ctx.current[1]) {
    case Quantifier.ONE: {
      const result = ctx.current[0].match(ctx.state, ctx.str, ctx.pos)
      if (result) ctx.advance(result)
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
      if (ctx.advanced && ctx.nextMatches()) {
        ctx.advance()
        break
      }
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
      ctx.advance()
      ctx.advanced = null
      break
    }

    case Quantifier.REPEATING_ONE_OR_MORE: {
      if (ctx.advanced && ctx.nextMatches()) {
        ctx.advance()
        break
      }
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

/** Utility function for running a rule as many times as possible. */
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

/** Types of quantifier for a chain rule. */
enum Quantifier {
  /** No suffix. */
  ONE,
  /** `?` suffix. */
  OPTIONAL,
  /** `*` suffix. */
  ZERO_OR_MORE,
  /** `+` suffix. */
  ONE_OR_MORE,
  /** Chain rule strings separated by `|` pipes. */
  ALTERNATIVES,
  /** Rule names separated by `|*` pipes. */
  REPEATING_ZERO_OR_MORE,
  /** Rule names separated by `|+` pipes. */
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

/** Class used for tracking the state of a in progress chain match. */
class ChainContext {
  /** The current {@link GrammarState}. */
  declare state: GrammarState

  /** The list of rules to match with. */
  declare rules: ChainRule[]

  /** The current string to match. */
  declare str: string

  /** The current position. */
  declare pos: number

  /** The totality of the string that has been matched so far. */
  declare total: string

  /** The list of results to be returned. */
  declare results: Matched[] | null

  /** The current rule index. */
  declare index: number

  /** If true, the current match has failed. */
  declare failed: boolean

  /** Used for keeping track of state with the `REPEATING` quantifiers. */
  declare advanced: boolean | null

  /**
   * A skip pattern to use.
   *
   * @see {@link Chain}
   */
  declare skipMatcher?: RegExpMatcher

  constructor(rules: ChainRule[], skip?: RegExpMatcher) {
    this.rules = rules
    this.clear()
    if (skip) this.skipMatcher = skip
  }

  /** True if the running match has finished. */
  get done() {
    return this.index >= this.rules.length
  }

  /** Gets the current rule, based on the current index. */
  get current() {
    if (this.done) throw new Error("Cannot get current rule when done")
    return this.rules[this.index]
  }

  /** Adds a {@link Matched} to the result list. */
  add(result: Matched) {
    if (!this.results) this.results = []
    this.results.push(result)
    this.total += result.total
    this.pos += result.length
  }

  /** Sets the match to have failed. */
  fail() {
    this.failed = true
    this.index = this.rules.length
  }

  /**
   * Advances to the next rule.
   *
   * @param result - A result to add to the results list, if desired.
   */
  advance(result?: Matched) {
    if (result) this.add(result)
    this.index++
  }

  /**
   * Finishes and cleans up. Returns `null` if the match failed, otherwise
   * a list of {@link Matched} objects will be returned.
   */
  finish() {
    if (this.failed) {
      this.clear()
      return null
    }
    const results = this.results
    this.clear()
    return results
  }

  /**
   * Checks to see if the next rule would match. Used for leaving
   * `REPEATING` quantifier rules early.
   */
  nextMatches() {
    if (this.done) throw new Error("Cannot get next rule when done")
    const rule = this.rules[this.index + 1]
    const state = this.state.clone()
    switch (rule[1]) {
      case Quantifier.ONE:
      case Quantifier.OPTIONAL:
      case Quantifier.ZERO_OR_MORE:
      case Quantifier.ONE_OR_MORE: {
        const result = rule[0].match(state, this.str, this.pos)
        if (result) return true
        break
      }
      case Quantifier.ALTERNATIVES: {
        for (let i = 0; i < rule[0].length; i++) {
          for (let j = 0; j < rule[0][i].length; j++) {
            const result = rule[0][i][0].match(state, this.str, this.pos)
            if (result) return true
          }
        }
        break
      }
      case Quantifier.REPEATING_ZERO_OR_MORE:
      case Quantifier.REPEATING_ONE_OR_MORE: {
        for (let i = 0; i < rule[0].length; i++) {
          const result = rule[0][i].match(state, this.str, this.pos)
          if (result) return true
        }
      }
    }
    return false
  }

  /** Greedy consumes any characters matched by the `skip` pattern. */
  skip() {
    if (!this.skipMatcher) return
    let result
    while ((result = this.skipMatcher.match(this.str, this.pos))) {
      this.pos += result.length
      this.total += result.total
    }
  }

  /** Clears out the current state. */
  private clear() {
    this.total = ""
    this.index = 0
    this.failed = false
    this.advanced = null
    this.results = null
  }

  /** Resets the current state with the new match arguments. */
  reset(state: GrammarState, str: string, pos: number) {
    this.state = state
    this.str = str
    this.pos = pos
  }
}

/**
 * Parses a chain rule string, and returns the rule(s) it specifies and
 * what type of quantifier it uses.
 */
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
