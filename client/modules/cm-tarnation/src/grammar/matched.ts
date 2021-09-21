import { Wrapping } from "../enums"
import type { GrammarToken, MatchOutput } from "../types"
import { Node } from "./node"
import type { GrammarState } from "./state"

/** Represents a leaf or branch of a tree of matches found by a grammar. */
export class Matched {
  /** The total length of the match. */
  declare length: number

  constructor(
    /** The current {@link GrammarState}. */
    public state: GrammarState,
    /** The match's {@link Node} type. */
    public node: Node,
    /** The entire matched string. */
    public total: string,
    /** The start of the match. */
    public from: number,
    /** The children contained by this match's {@link Node}. */
    public captures?: Matched[],
    /**
     * The wrapping mode of the match. There are three modes:
     *
     * - FULL: The {@link Node} in this match contains the entirety of the branch.
     * - BEGIN: The {@link Node} in this match begins the branch.
     * - END: The {@link Node} in this match ends the branch.
     */
    public wrapping: Wrapping = Wrapping.FULL
  ) {
    this.length = total.length
  }

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

  /**
   * Wraps this `Matched` with another one.
   *
   * @param node - The node of the `Matched` to wrap with.
   * @param wrap - The wrapping mode, if different.
   */
  wrap(node: Node, wrap = this.wrapping) {
    return new Matched(this.state, node, this.total, this.from, [this], wrap)
  }

  push(node: Node, side: -1 | 1, wrap = this.wrapping) {
    if (wrap === Wrapping.FULL) throw new Error("Cannot push onto a FULL match")
    this.captures ??= []
    let pos = side === -1 ? this.from : this.from + this.length
    const match = new Matched(this.state, node, "", pos, undefined, wrap)
    if (side === -1) this.captures.unshift(match)
    else this.captures.push(match)
  }

  /** Returns this match represented as a raw {@link MatchOutput}. */
  output(): MatchOutput {
    let captures: string[] | null = null
    if (this.captures) {
      captures = []
      for (let i = 0; i < this.captures.length; i++) {
        captures.push(this.captures[i].total)
      }
    }
    return { total: this.total, captures, length: this.length }
  }

  /** Internal method for compiling. */
  private _compile() {
    if (!this.captures) return compileLeaf(this)

    // verbose approach for performance
    const tokens: GrammarToken[] = []
    for (let i = 0; i < this.captures!.length; i++) {
      const compiled = this.captures![i]._compile()
      // wasn't emitted
      if (!compiled) continue
      // leaf
      if (!isGrammarTokenList(compiled)) tokens.push(compiled)
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
    if (isGrammarTokenList(compiled)) return compiled
    else if (compiled) return [compiled]
    else return []
  }
}

function isGrammarTokenList(
  token: GrammarToken | GrammarToken[]
): token is GrammarToken[] {
  if (token.length === 0) return true
  return Array.isArray(token[0])
}

/** Compiles a {@link Matched} as a leaf. */
function compileLeaf(match: Matched): GrammarToken {
  if (match.wrapping !== Wrapping.FULL && match.node === Node.None) {
    throw new Error("Cannot compile a null leaf with a non-full wrapping")
  }

  // prettier-ignore
  switch(match.wrapping) {
    case Wrapping.FULL: return [
      match.node === Node.None ? null : match.node.id,
      match.from,
      match.from + match.length
    ]

    case Wrapping.BEGIN: return [
      null,
      match.from,
      match.from + match.length,
      [match.node.id]
    ]

    case Wrapping.END: return [
      null,
      match.from,
      match.from + match.length,
      undefined,
      [match.node.id]
    ]
  }
}

/**
 * Compiles a {@link Matched} as a tree, with the given token list being its
 * already compiled children.
 */
function compileTree(match: Matched, tokens: GrammarToken[]) {
  if (match.node === Node.None) return tokens

  const first = tokens[0]
  const last = tokens[tokens.length - 1]

  if (match.wrapping === Wrapping.FULL || match.wrapping === Wrapping.BEGIN) {
    first[3] ??= []
    first[3].unshift(match.node.id)
  }

  if (match.wrapping === Wrapping.FULL || match.wrapping === Wrapping.END) {
    last[4] ??= []
    last[4].push(match.node.id)
  }

  return tokens
}
