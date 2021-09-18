import type * as DF from "../definition"
import { Matched } from "../matched"
import { RegExpMatcher } from "../matchers/regexp"
import { Node } from "../node"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import { Rule } from "./rule"

export class Chain extends Rule {
  private declare chain: ReturnType<typeof parseChainItem>[]
  private declare skip?: RegExpMatcher

  constructor(repo: Repository, rule: DF.Chain) {
    super(repo, rule)
    if (rule.skip) {
      this.skip = new RegExpMatcher(rule.skip, repo.ignoreCase, repo.variables)
    }
    this.chain = rule.chain.map(item => parseChainItem(repo, item, this.skip))
  }

  // TODO: can this be cleaned up?
  // it's fast, but it's not very readable

  exec(state: GrammarState, str: string, pos: number) {
    const from = pos
    let total = ""
    let iter: Iterable<Matched | null> | null = null
    const results: Matched[] = []
    chain: for (let i = 0; i < this.chain.length; i++) {
      // check skip rule, and if it passes, skip forward a position
      if (this.skip) {
        let result
        while ((result = this.skip.match(str, pos))) {
          total += result.total
          pos += result.length
        }
      }

      if (!iter) iter = this.chain[i](state, str, pos)
      let repeating = false
      for (const result of iter) {
        if (!result) return null
        results.push(result)

        total += result.total
        pos += result.length

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
    return new Matched(state, this.node, total, from, results)
  }
}

function parseChainItem(repo: Repository, str: string, skip?: RegExpMatcher) {
  const repeatAlternatives = /\|[*+]/.test(str)
  const normalAlternatives = /\|(?![*+])/.test(str)

  if (repeatAlternatives && normalAlternatives) {
    throw new Error("Cannot have repeating alternative and non-repeating alternatives")
  }

  if (!repeatAlternatives && !normalAlternatives) {
    return parseChainRule(repo, str, skip)
  }

  const rules = str.split(/\s*\|[*+]?\s*/).map(item => parseChainRule(repo, item, skip))

  function* iterate(state: GrammarState, str: string, pos: number) {
    let maybeFailed = false
    let advanced = false
    rules: for (let i = 0; i < rules.length; i++) {
      // check skip rule, and if it passes, skip forward a position
      if (skip) {
        let result
        while ((result = skip.match(str, pos))) {
          yield new Matched(state, Node.None, result.total, pos)
          pos += result.length
        }
      }
      for (const result of rules[i](state, str, pos)) {
        if (!result) {
          maybeFailed = true
          break
        } else {
          advanced = true
          pos += result.length
          yield result
        }
        if (advanced) break rules
      }
    }
    if (maybeFailed && !advanced) yield null
  }

  if (repeatAlternatives) {
    const zeroOrMore = /\|\*/.test(str)
    const oneOrMore = /\|\+/.test(str)

    if (zeroOrMore && oneOrMore) {
      throw new Error("Cannot have repeating alternatives with both * and +")
    }

    if (zeroOrMore) {
      return function* (state: GrammarState, str: string, pos: number) {
        let result: void | Matched | null
        while ((result = iterate(state, str, pos).next().value)) {
          yield result
          pos += result.length
        }
      }
    } else {
      return function* (state: GrammarState, str: string, pos: number) {
        let advanced = false
        let result: void | Matched | null
        while ((result = iterate(state, str, pos).next().value)) {
          yield result
          advanced = true
          pos += result.length
        }
        if (!advanced) yield null
      }
    }
  } else if (normalAlternatives) {
    return iterate
  }

  throw new Error("Unreachable")
}

function parseChainRule(repo: Repository, str: string, skip?: RegExpMatcher) {
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
            pos += result.length
            if (skip) {
              let result
              while ((result = skip.match(str, pos))) {
                yield new Matched(state, Node.None, result.total, pos)
                pos += result.length
              }
            }
          }
        }
      }
    }
    case ChainRuleType.ONE_OR_MORE: {
      return function* (state: GrammarState, str: string, pos: number) {
        let advanced = false
        let result
        while ((result = rule.match(state, str, pos))) {
          if (advanced && !result) {
            yield null
            break
          }
          yield result
          advanced = true
          pos += result.length
          if (skip) {
            let result
            while ((result = skip.match(str, pos))) {
              yield new Matched(state, Node.None, result.total, pos)
              pos += result.length
            }
          }
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
