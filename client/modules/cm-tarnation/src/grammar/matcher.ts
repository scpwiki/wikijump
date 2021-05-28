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
  | { matcher: number[];         type: MatcherType.Points; src: string, cased: boolean }

export class Matcher {
  private declare elements?: MatcherElement[]
  private declare ignoreCase: boolean

  constructor(grammar: Grammar, matchers: DF.Match) {
    if (!isArray(matchers)) matchers = [matchers]

    this.ignoreCase = grammar.ignoreCase

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
        const src = String.fromCodePoint(...(matcher as number[]))
        const cased = this.ignoreCase ? src.toLowerCase() !== src.toUpperCase() : false
        return { matcher, type, src, cased } as MatcherElement
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

  private static match(
    element: MatcherElement,
    cx: GrammarContext,
    str: string,
    pos: number,
    ignoreCase: boolean
  ): { total: string | null; match: string[] | null } {
    let total: string | null = null
    let match: string[] | null = null

    switch (element.type) {
      case MatcherType.Function: {
        match = element.matcher(cx, str, pos)
        break
      }

      case MatcherType.RegExp: {
        // what follows looks really dumb, but is actually _way faster_.
        // for some reason, if you _test_ the regex first, and then do exec,
        // that's a lot faster. I don't know why, I don't get it - but whatever.
        // FYI by "a lot faster" we're talking 2x-3x speedup.
        element.matcher.lastIndex = pos
        if (element.matcher.test(str)) {
          element.matcher.lastIndex = pos
          const result = element.matcher.exec(str)
          if (result) {
            total = result.shift()!
            if (result.length) match = result
          }
        }
        break
      }

      case MatcherType.Substitute: {
        const sub = Grammar.sub(cx, element.matcher)
        if (!sub) break
        if (pointsMatch(toPoints(sub), str, pos)) match = [sub]
        break
      }

      case MatcherType.Points: {
        if (str.length < element.matcher.length) break
        if (ignoreCase && element.cased) {
          const against = str.substr(pos, element.matcher.length).toLowerCase()
          if (pointsMatch(element.matcher, against, 0)) {
            match = [element.src]
          }
        } else if (pointsMatch(element.matcher, str, pos)) {
          match = [element.src]
        }
        break
      }
    }

    return { total, match }
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

    // just shortcut out if we have only one matcher
    if (this.elements.length === 1) {
      const element = this.elements[0]
      const { total, match } = Matcher.match(element, cx, str, pos, this.ignoreCase)
      if (total || match) {
        return match ? [total ?? match.join(""), ...match] : [total!]
      }
      return null
    }

    const found: string[] = [""]
    const start = pos

    // for loop is faster and doesn't invoke iterator methods needlessly
    for (let i = 0; i < this.elements.length; i++) {
      const element = this.elements[i]
      const { total, match } = Matcher.match(element, cx, str, pos, this.ignoreCase)
      if (!total && !match) return null

      if (total) found[0] += total

      if (match?.length) {
        if (!total) found[0] += match.join("")
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
