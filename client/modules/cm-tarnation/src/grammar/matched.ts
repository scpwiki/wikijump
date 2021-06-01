import { klona } from "klona"
import { hasSigil, isEmpty } from "wj-util"
import { Action, ActionMode } from "./action"
import type * as DF from "./definition"
import { Grammar, GrammarMatchState, GrammarToken } from "./grammar"

export function createToken({ from, to, action, context, state }: Matched): GrammarToken {
  let { type, open, close, next, switchTo, embedded } = action

  if (state) {
    if (hasSigil(next, ["$", "::"])) next = Grammar.sub(state, next)
    if (hasSigil(switchTo, ["$", "::"])) switchTo = Grammar.sub(state, switchTo)
    if (hasSigil(embedded, ["$", "::"])) embedded = Grammar.sub(state, embedded)
  }

  if (context && isEmpty(context)) context = undefined

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
export function wrapTokens(tokens: GrammarToken[], { context, state, action }: Matched) {
  const first = tokens[0]
  const last = tokens[tokens.length - 1]

  let { type, mode, next, switchTo, open, close, embedded } = action

  if (context) last.context = { ...last.context, ...context }

  if (state) {
    if (hasSigil(next, ["$", "::"])) next = Grammar.sub(state, next)
    if (hasSigil(switchTo, ["$", "::"])) switchTo = Grammar.sub(state, switchTo)
    if (hasSigil(embedded, ["$", "::"])) embedded = Grammar.sub(state, embedded)
  }

  if (next || switchTo) {
    tokens.unshift(
      createToken({
        from: last.to,
        to: last.to,
        action: { type: "", next, switchTo },
        state
      } as Matched)
    )
  }

  if (embedded && !embedded.endsWith("!")) {
    if (embedded === "@pop") {
      first.embedded = embedded
      first.empty = false
    } else {
      last.embedded = embedded
      last.empty = false
    }
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
    first.empty = false
    last.empty = false
  }

  return tokens
}

export class Matched {
  declare total: string
  declare action: Action
  declare captures: Set<Matched>
  declare size: number
  declare length: number
  declare from: number
  declare to: number
  declare state?: GrammarMatchState
  declare context?: DF.Context

  constructor(
    total: string,
    action: Action,
    offset: number,
    state?: GrammarMatchState,
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
    this.state = state
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
