import { NodeProp, NodePropSource, NodeType, SyntaxNode } from "@lezer/common"
import {
  continuedIndent,
  delimitedIndent,
  EditorState,
  flatIndent,
  foldInside,
  foldNodeProp,
  indentNodeProp,
  styleTags,
  Tag,
  tags,
  TreeIndentContext
} from "@wikijump/codemirror/cm"
import { EmbeddedParserProp, re } from "./../util"
import type * as DF from "./definition"

/** Effectively a light wrapper around a CodeMirror `NodeType`. */
export class Node {
  /** The unique ID for this node. */
  declare id: number

  /** The name of this node, which may be different to what it emits in the AST. */
  declare name: string

  /** The `NodeType` used by CodeMirror. */
  declare type: NodeType

  /** @param id - The ID to assign to this node. */
  constructor(
    id: number,
    { type, emit, tag, openedBy, closedBy, group, nest, fold, indent }: DF.Node
  ) {
    if (!type) throw new Error("Node name/type is required")
    if (emit === false) throw new Error("Node cannot be emitted")

    this.id = id
    this.name = type

    if (typeof emit !== "string") emit = type

    const props: NodePropSource[] = []

    // prettier-ignore
    {
      if (tag)      props.push(styleTags(parseTag(emit, tag)))
      if (nest)     props.push(EmbeddedParserProp.add({ [emit]: nest                }))
      if (openedBy) props.push(NodeProp.openedBy .add({ [emit]: [openedBy].flat()   }))
      if (closedBy) props.push(NodeProp.closedBy .add({ [emit]: [closedBy].flat()   }))
      if (group)    props.push(NodeProp.group    .add({ [emit]: [group].flat()      }))
      if (fold)     props.push(foldNodeProp      .add({ [emit]: parseFold(fold)     }))
      if (indent)   props.push(indentNodeProp    .add({ [emit]: parseIndent(indent) }))
    }

    this.type = NodeType.define({ id, name: emit, props })
  }

  /** Special `Node` used for when a rule doesn't emit anything. */
  static None = new Node(-1, { type: "_none", emit: "None" })
}

/**
 * 1. Tag modifier text
 * 2. Tag function name
 * 3. Tag function argument
 * 4. Tag name, no function
 */
const PARSE_TAG_REGEX = /^(?:\((\S*?)\))?(?:\s+|^)(?:(?:(\S+?)\((\S+)\))|(\S+))$/

/**
 * Parses a tag string, and converts it into an object that can be fed into
 * CodeMirror's `styleTags` function.
 *
 * Examples:
 *
 * ```text
 * tag
 * func(tag)
 * (!) tag
 * (!) func(tag)
 * (...) tag
 * (...) func(tag)
 * (parent/) tag
 * (parent/) func(tag)
 * (grandparent/parent) tag
 * (grandparent/parent) func(tag)
 * ```
 */
function parseTag(node: string, str: DF.Tag) {
  const [, modifier, func, arg, last] = PARSE_TAG_REGEX.exec(str)!

  if (last && !(last in tags)) throw new Error(`Unknown tag: ${last}`)
  if (func && !(func in tags)) throw new Error(`Unknown tag function: ${func}`)
  if (arg && !(arg in tags)) throw new Error(`Unknown tag argument: ${arg}`)

  let name = arg ? arg : last
  let prefix = ""
  let suffix = ""

  // @ts-ignore TS doesn't realize I've checked for this
  let tag: Tag = tags[name]

  // @ts-ignore ditto
  if (func) tag = tags[func](tag)

  if (modifier) {
    if (modifier.endsWith("...")) suffix = "/..."
    if (modifier.endsWith("!")) suffix = "!"
    if (modifier.endsWith("/")) prefix = modifier
    // check for parents
    else {
      const split = modifier.split("/")
      const last = split[split.length - 1]
      if (last === "..." || last === "!") split.pop()
      if (split.length) prefix = `${split.join("/")}/`
    }
  }

  // e.g. foo/... or foo/bar/... etc.
  const style = `${prefix}${node}${suffix}`

  return { [style]: tag }
}

/**
 * `offset(n n)`, `offset(-2 -5)`, `offset(+1 2)`, `offset(0 0)`, etc.
 *
 * 1. Left offset
 * 2. Right offset
 */
const PARSE_OFFSET_FOLD_REGEX = /^offset\(([+-]?\d+),\s+([+-]?\d+)\)$/

/** Parses a `fold` string, returning a CodeMirror `foldNodeProp` compatible function. */
function parseFold(
  fold: true | string
): (node: SyntaxNode, state: EditorState) => { from: number; to: number } | null {
  // prettier-ignore
  switch (fold) {
    // folds entire node
    case true: return node => ({ from: node.from, to: node.to })
    // folds between two delimiters, which are the first and last child
    case "inside": return foldInside
    // folds everything past the first-ish line
    case "past_first_line": return (node, state) => ({
      from: Math.min(node.from + 20, state.doc.lineAt(node.from).to),
      to: node.to - 1
    })
    // like the "true" case, except with an offset
    // (or the fold string is invalid)
    default: {
      if (fold.startsWith("offset")) {
        const match = PARSE_OFFSET_FOLD_REGEX.exec(fold)
        if (!match) throw new Error("Invalid fold offset")
        const left = parseInt(match[1], 10)
        const right = parseInt(match[2], 10)
        return node => ({ from: node.from + left, to: node.to + right })
      } else {
        throw new Error(`Unknown fold option: ${fold}`)
      }
    }
  }
}

/** 1. Closing */
const PARSE_DELIMITED_INDENT_REGEX = /^delimited\((.+?)\)$/

/** 1. Except Regex */
const PARSE_CONTINUED_INDENT_REGEX = /^continued(?:\((.+?)\))?$/

/** 1. Units */
const PARSE_ADD_INDENT_REGEX = /^add\(([+-]?\d+)\)$/

/** 1. Units */
const PARSE_SET_INDENT_REGEX = /^set\(([+-]?\d+)\)$/

/** Parses an indent string, returning a `indentNodeProp` compatible function. */
function parseIndent(indent: string): (context: TreeIndentContext) => number {
  if (indent === "flat") return flatIndent
  if (indent === "continued") return continuedIndent()

  if (indent.startsWith("delimited")) {
    const match = PARSE_DELIMITED_INDENT_REGEX.exec(indent)
    if (!match) throw new Error("Invalid delimited indent")
    const [, closing] = match
    return delimitedIndent({ closing })
  }

  if (indent.startsWith("continued")) {
    const match = PARSE_CONTINUED_INDENT_REGEX.exec(indent)
    if (!match) throw new Error("Invalid continued indent")
    const except = re(match[1])
    if (!except) throw new Error("Invalid continued indent except regex")
    return continuedIndent({ except })
  }

  if (indent.startsWith("add")) {
    const match = PARSE_ADD_INDENT_REGEX.exec(indent)
    if (!match) throw new Error("Invalid add indent")
    const units = parseInt(match[1], 10)
    return cx => cx.baseIndent + cx.unit * units
  }

  if (indent.startsWith("set")) {
    const match = PARSE_SET_INDENT_REGEX.exec(indent)
    if (!match) throw new Error("Invalid set indent")
    const units = parseInt(match[1], 10)
    return () => units
  }

  throw new Error(`Unknown indent option: ${indent}`)
}
