import { createID } from "@wikijump/util"
import type * as DF from "../definition"
import { createLookbehind, re } from "../helpers"
import { Matched } from "../matched"
import { RegExpMatcher } from "../matchers/regexp"
import { Node } from "../node"
import type { Repository } from "../repository"
import type { GrammarState } from "../state"
import type { MatchOutput } from "../types"

export abstract class Rule {
  declare name: string
  declare node: Node
  declare lookbehind?: (str: string, pos: number) => boolean
  declare lookahead?: RegExpMatcher
  declare contextSetters?: ((state: GrammarState) => void)[]
  declare captures: (Node | CaptureFunction)[]
  declare rematch?: boolean

  constructor(repo: Repository, rule: DF.Rule) {
    let type = rule.type ?? createID()
    let emit = rule.type && rule.emit !== false

    this.name = type
    this.node = !emit ? Node.None : new Node(repo.id(), rule)

    if (rule.captures) {
      this.captures = []
      for (const key in rule.captures) {
        const value = rule.captures[key]
        const idx = parseInt(key, 10)
        // conditional
        if ("matches" in value) this.captures[idx] = captureFunction(repo, value)
        // node or reused
        else this.captures[idx] = repo.add(value)
      }
    }

    if (rule.lookbehind) {
      const negative = rule.lookbehind[0] === "!"
      const regexp = re(rule.lookbehind)
      if (!regexp) throw new Error("Invalid lookbehind regexp")
      this.lookbehind = createLookbehind(regexp, negative)
    }

    if (rule.lookahead) {
      this.lookahead = new RegExpMatcher(rule.lookahead, repo.ignoreCase, repo.variables)
    }

    if (rule.context) {
      if (!Array.isArray(rule.context)) {
        this.contextSetters = [contextSetter(rule.context)]
      } else {
        this.contextSetters = rule.context.map(contextSetter)
      }
    }

    if (rule.rematch) this.rematch = true
  }

  abstract exec(
    state: GrammarState,
    str: string,
    pos: number
  ): Matched | MatchOutput | null

  match(state: GrammarState, str: string, pos: number): Matched | null {
    if (this.lookbehind && !this.lookbehind(str, pos)) return null

    let matched = this.exec(state, str, pos)

    if (matched) {
      if (this.lookahead && !this.lookahead?.test(str, pos + matched.length)) {
        return null
      }

      // if rematch, we just return basically "success!",
      // and let the tokenizer move on
      if (this.rematch) return new Matched(state, this.node, "", pos)

      // returned a MatchOutput, which we need to turn into a Matched
      if (!(matched instanceof Matched)) {
        const output = matched
        matched = new Matched(state, this.node, matched.total, pos)

        state.last = output

        if (this.captures) {
          if (!output.captures) throw new Error("Output has no captures when it should")

          const captures: Matched[] = []
          let capturePos = pos

          for (let i = 0; i < this.captures.length; i++) {
            const val = this.captures[i]
            const capture = output.captures[i]

            let node = Node.None

            if (val) {
              const resolved = typeof val === "function" ? val(state, capture) : val
              if (resolved === false) return null
              if (resolved === true) node = Node.None
              else node = resolved
            }

            captures.push(new Matched(state, node, capture, capturePos))

            capturePos += capture.length
          }

          matched.captures = captures
        }
      }
      // returned a full Matched, which we may need to wrap
      else {
        state.last = matched.output()

        if (this.captures) {
          for (let i = 0; i < this.captures.length; i++) {
            const val = this.captures[i]
            const capture = matched?.captures?.[i]

            if (!capture) throw new Error("Missing capture")

            let node: Node | null = null

            if (val) {
              const resolved = typeof val === "function" ? val(state, capture.total) : val
              if (resolved === false) return null
              if (resolved === true) continue
              else node = resolved
            }

            if (node) matched.captures![i] = capture.wrap(node)
          }
        }
      }

      if (this.contextSetters) {
        for (let i = 0; i < this.contextSetters.length; i++) {
          this.contextSetters[i](state)
        }
      }
    }

    return matched
  }
}

function contextSetter(setter: DF.ContextSetter) {
  return (state: GrammarState) => {
    // check if and match conditions
    // if only "if", check if that string isn't empty
    // if only "matches", check against the entire match
    // if both, check "if" against "matches"
    if (setter.if || setter.matches) {
      const against = setter.if ? state.sub(setter.if) : state.last?.total

      if (typeof against !== "string") return

      if (!setter.matches && !setter.if) return

      if (setter.matches) {
        const matches = state.sub(setter.matches)
        if (matches instanceof RegExp) {
          if (!matches.test(against)) return
        } else if (typeof matches === "string") {
          if (matches !== against) return
        } else {
          return
        }
      }
    }

    // if we're here, we can set context
    state.set(setter.set, setter.to)
  }
}

type CaptureFunction = (state: GrammarState, capture: string) => Node | boolean

function captureFunction(repo: Repository, cond: DF.CaptureCondition): CaptureFunction {
  // TODO: fix lower casing matching here

  const matcher = cond.matches.startsWith("/") ? re(cond.matches) : cond.matches

  if (!matcher) throw new Error("Invalid match condition")

  let nodeThen: Node | null = null
  let nodeElse: Node | null = null

  if (cond.then) nodeThen = repo.add(cond.then)
  if (cond.else) nodeElse = repo.add(cond.else)

  return (state: GrammarState, capture: string) => {
    const matches = typeof matcher === "string" ? state.sub(matcher) : matcher
    if (typeof matches !== "string" && !(matches instanceof RegExp)) {
      throw new Error("Invalid match condition")
    }

    const against = cond.if ? state.sub(cond.if) : capture
    if (typeof against !== "string") throw new Error("Invalid capture condition")

    let passed = false

    if (matches instanceof RegExp) {
      if (matches.test(against)) passed = true
    } else if (typeof matches === "string") {
      if (matches === against) passed = true
    }

    if (passed && nodeThen) return nodeThen
    if (!passed && nodeElse) return nodeElse
    return passed
  }
}
