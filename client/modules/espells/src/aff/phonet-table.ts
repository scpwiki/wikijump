import { re } from "../util"

// 1. letters, 2. optional, 3. lookahead, 4. flags, 5. priority
const RULE_PATTERN = /^(\p{L}+)(?:\((\p{L}+)\))?(-+)?([\^$<]*)(\d)?$/u

export class PhonetTable {
  rules: Record<string, PhonetTableRule[]> = {}

  constructor(table: [string, string][]) {
    for (const [search, replacement] of table) {
      const match = RULE_PATTERN.exec(search)
      if (!match) throw new SyntaxError(`Invalid PhonetTable pattern '${search}'`)

      const [letters, optional, lookahead, flags, priority] = match

      let text = [...letters]
      if (optional) text.push(`[${optional}]`)

      let regex: RegExp
      if (lookahead) {
        const la = lookahead.length
        regex = re`/${text.slice(0, -la).join("")}(?=${text.slice(-la).join("")})/`
      } else {
        regex = re`/${text.join("")}/`
      }

      this.rules[search[0]] ??= []
      this.rules[search[0]].push(
        new PhonetTableRule(
          regex,
          replacement,
          flags?.includes("^"),
          flags?.includes("$"),
          Boolean(lookahead),
          priority ? parseInt(priority) : 5
        )
      )
    }
  }
}

class PhonetTableRule {
  constructor(
    public search: RegExp,
    public replacement: string,
    public start = false,
    public end = false,
    public followup = true,
    public priority = 5
  ) {}

  match(word: string, pos: number) {
    if (this.start && pos > 0) return false
    this.search.lastIndex = pos
    const match = this.search.exec(word)
    if (match) {
      if (this.end) {
        return match[0].length !== word.length ? false : match
      } else {
        return match
      }
    }
    return false
  }
}
