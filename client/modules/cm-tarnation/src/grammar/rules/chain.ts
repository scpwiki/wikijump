import type * as DF from "../definition"
import { Matched } from "../matched"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Wrapping } from "../types"
import { Rule } from "./rule"

export class Chain extends Rule {
  private declare chain: ReturnType<typeof parseChainItem>[]

  constructor(repo: Repository, rule: DF.Chain) {
    super(repo, rule)
    this.chain = rule.chain.map(item => parseChainItem(repo, item))
  }

  // TODO: can this be cleaned up?
  // it's fast, but it's not very readable

  exec(state: GrammarState, str: string, pos: number) {
    const from = pos
    let total = ""
    let iter: Iterable<Matched | null> | null = null
    const results: Matched[] = []
    chain: for (let i = 0; i < this.chain.length; i++) {
      if (!iter) iter = this.chain[i](state, str, pos)
      let repeating = false
      for (const result of iter) {
        if (!result) return null
        results.push(result)

        total += result.total
        pos += result.total.length

        // if we're in a repeating rule (more than one match)
        // we need to check the next rule, and if it matches
        // we need to continue with the next iterator
        if (repeating && this.chain[i + 1]) {
          const next = this.chain[i + 1](state, str, pos)
          const nextResult = next.next().value
          if (nextResult) {
            results.push(nextResult)
            iter = next
            continue chain
          }
        }

        repeating = true
      }

      iter = null
    }
    if (!results.length) return null
    return new Matched(state, this.node, total, from, Wrapping.FULL, results)
  }
}

function parseChainItem(repo: Repository, str: string) {
  const split = str.split(/\s*\|\s*/)
  if (split.length === 1) return parseChainRule(repo, str)

  const rules = split.map(item => parseChainRule(repo, item))

  const repeating = split.every(
    item => item[item.length - 1] === "+" || item[item.length - 1] === "*"
  )

  function* iterate(state: GrammarState, str: string, pos: number) {
    let maybeFailed = false
    let advanced = false
    for (let i = 0; i < rules.length; i++) {
      for (const result of rules[i](state, str, pos)) {
        if (!result) {
          maybeFailed = true
          break
        } else {
          advanced = true
          pos += result.total.length
          yield result
        }
      }
    }
    if (maybeFailed && !advanced) yield null
  }

  return function* (state: GrammarState, str: string, pos: number) {
    if (repeating) {
      let result: void | Matched | null
      while ((result = iterate(state, str, pos).next().value) !== undefined) {
        yield result
        if (!result) return
        pos += result.total.length
      }
    } else {
      yield* iterate(state, str, pos)
    }
  }
}

function parseChainRule(repo: Repository, str: string) {
  let type = ChainRuleType.ONE
  // prettier-ignore
  switch (str[str.length - 1]) {
    case "?": type = ChainRuleType.OPTIONAL; break
    case "*": type = ChainRuleType.ZERO_OR_MORE; break
    case "+": type = ChainRuleType.ONE_OR_MORE; break
  }

  if (type !== ChainRuleType.ONE) {
    str = str.slice(0, str.length - 1)
  }

  const rule = repo.get(str)

  if (!(rule instanceof Rule)) throw new Error(`Rule "${str}" not found`)

  switch (type) {
    case ChainRuleType.ONE: {
      return function* (state: GrammarState, str: string, pos: number) {
        yield rule.match(state, str, pos)
      }
    }
    case ChainRuleType.OPTIONAL: {
      return function* (state: GrammarState, str: string, pos: number) {
        const result = rule.match(state, str, pos)
        if (result) yield result
      }
    }
    case ChainRuleType.ZERO_OR_MORE: {
      return function* (state: GrammarState, str: string, pos: number) {
        let result
        while ((result = rule.match(state, str, pos))) {
          if (result) {
            yield result
            pos += result.total.length
          }
        }
      }
    }
    case ChainRuleType.ONE_OR_MORE: {
      return function* (state: GrammarState, str: string, pos: number) {
        let advanced = false
        let result
        while ((result = rule.match(state, str, pos))) {
          if (advanced && !result) return
          yield result
          advanced = true
          pos += result.total.length
        }
      }
    }
  }
}

enum ChainRuleType {
  ONE,
  OPTIONAL,
  ZERO_OR_MORE,
  ONE_OR_MORE
}
