import { Node } from "./node"
import type { GrammarState } from "./state"
import { GrammarToken, Inclusivity, MatchOutput, Nesting, Wrapping } from "./types"

/** Represents a leaf or branch of a tree of matches found by a grammar. */
export class Matched {
  constructor(
    /** The current {@link GrammarState}. */
    public state: GrammarState,
    /** The match's {@link Node} type. */
    public node: Node,
    /** The entire matched string. */
    public total: string,
    /** The start of the match. */
    public from: number,
    /**
     * The wrapping mode of the match. There are three modes:
     *
     * - FULL: The {@link Node} in this match contains the entirety of the branch.
     * - BEGIN: The {@link Node} in this match begins the branch.
     * - END: The {@link Node} in this match ends the branch.
     */
    public wrapping: Wrapping = Wrapping.FULL,
    /** The children contained by this match's {@link Node}. */
    public captures?: Matched[]
  ) {}

  /** Changes the starting offset of the match. */
  offset(offset: number) {
    if (this.captures) {
      for (let i = 0; i < this.captures.length; i++) {
        const child = this.captures[i]
        child.offset(child.from - this.from + offset)
      }
    }
    this.from = offset
  }

  wrap(node: Node, wrap = this.wrapping) {
    return new Matched(this.state, node, this.total, this.from, wrap, [this])
  }

  output(): MatchOutput {
    let captures: string[] | null = null
    if (this.captures) {
      captures = []
      for (let i = 0; i < this.captures.length; i++) {
        captures.push(this.captures[i].total)
      }
    }
    return { total: this.total, captures, length: this.total.length }
  }

  /** Internal method for compiling. */
  private _compile() {
    if (!this.captures) {
      if (this.node !== Node.None) return compileLeaf(this)
      return null
    }

    // verbose approach for performance
    const tokens: GrammarToken[] = []
    for (let i = 0; i < this.captures!.length; i++) {
      const compiled = this.captures![i]._compile()
      // wasn't emitted
      if (!compiled) continue
      // leaf
      if (!Array.isArray(compiled)) tokens.push(compiled)
      // branch
      else {
        for (let i = 0; i < compiled.length; i++) {
          tokens.push(compiled[i])
        }
      }
    }

    return compileTree(this, tokens)
  }

  /**
   * Compiles this match into a list of tokens. Always returns a list, even
   * if this match represents a leaf.
   */
  compile() {
    const compiled = this._compile()
    if (Array.isArray(compiled)) return compiled
    else if (compiled) return [compiled]
    else return []
  }
}

/** Compiles a {@link Matched} as a leaf. */
function compileLeaf(match: Matched): GrammarToken {
  const token: GrammarToken = {
    id: match.node.id,
    from: match.from,
    to: match.from + match.total.length
  }

  if (match.node.nest) {
    const lang = match.state.sub(match.node.nest)
    if (typeof lang !== "string") throw new Error("node.nest resolved badly")
    token.nest = `${lang}!` // "!" signifies nesting in a leaf
  }

  return token
}

/**
 * Compiles a {@link Matched} as a tree, with the given token list being its
 * already compiled children.
 */
function compileTree(match: Matched, tokens: GrammarToken[]) {
  if (match.node === Node.None) return tokens

  const first = tokens[0]
  const last = tokens[tokens.length - 1]

  let nest: string | null = null

  if (match.node.nest) {
    const lang = match.state.sub(match.node.nest)
    if (typeof lang !== "string") throw new Error("node.nest resolved badly")
    nest = lang
  }

  if (match.wrapping === Wrapping.FULL || match.wrapping === Wrapping.BEGIN) {
    first.open ??= []
    first.open.unshift([match.node.id, Inclusivity.INCLUSIVE])
    if (nest) first.nest = nest
  }

  if (match.wrapping === Wrapping.FULL || match.wrapping === Wrapping.END) {
    last.close ??= []
    last.close.push([match.node.id, Inclusivity.INCLUSIVE])
    if (nest) last.nest = Nesting.POP
  }

  return tokens
}
