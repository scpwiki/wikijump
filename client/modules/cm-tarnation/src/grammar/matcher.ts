import { isArray, isFunction, isRegExp } from "is-what"
import { hasSigil, pointsMatch, toPoints } from "wj-util"
import type * as DF from "./definition"
import { Grammar, GrammarContext } from "./grammar"

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

  constructor(private grammar: Grammar, matchers: DF.Match) {
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
      if (ignoreCase) matcher = matcher.toLowerCase()
      return toPoints(Grammar.expand(variables, matcher))
    }
  }

  exec(cx: GrammarContext, str: string, pos: number): string[] | null {
    if (!this.elements) return null

    if (cx.target && cx.last) {
      const { state, last, target, context } = cx
      if (hasSigil(target, "$")) {
        if (target !== "$#") {
          if (target === "$S") str = state
          else str = last[parseInt(target.slice(1))] ?? ""
        }
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
          if (cx.target === "$#") console.log(element.matcher)
          if (this.grammar.ignoreCase) {
            const against = str.slice(pos, str.length).toLowerCase()
            if (pointsMatch(element.matcher, against, 0)) {
              match = [element.source]
            }
          } else if (pointsMatch(element.matcher, str, pos)) {
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
