import type { SyntaxNode } from "@lezer/common"
import {
  syntaxTree,
  type Completion,
  type CompletionContext,
  type CompletionResult
} from "@wikijump/codemirror/cm"
import { BlockMap, BlockSet, ModuleMap, ModuleSet } from "./data/data"
import { htmlEnumCompletions } from "./data/html-attributes"

const blocksAutocompletion: Completion[] = Array.from(BlockSet).flatMap(
  block => block.completions
)

// we're also going to push special completions for "module" and "include"
blocksAutocompletion.push(
  {
    label: "module",
    type: "keyword"
  },
  {
    label: "include",
    type: "keyword"
  }
)

const moduleAutocompletion: Completion[] = Array.from(ModuleSet).flatMap(
  module => module.completions
)

export function completeFTML(context: CompletionContext): CompletionResult | null {
  const { state, pos } = context
  const around = syntaxTree(state).resolve(pos)
  const tree = around.resolve(pos, -1)

  const text = (tree: SyntaxNode | null | undefined) => {
    if (!tree) return null
    return context.state.sliceDoc(tree.from, tree.to)
  }

  // tag names
  if (tree.name === "BlockNameUnknown" || tree.name === "BlockName") {
    return { from: tree.from, to: pos, options: blocksAutocompletion }
  }

  // module names
  else if (tree.name === "ModuleName" || tree.name === "ModuleNameUnknown") {
    return { from: tree.from, to: pos, options: moduleAutocompletion }
  }

  // block node arguments
  else if (tree.name === "BlockLabel") {
    const tag = text(around.getChild("BlockName"))
    const module = text(
      around.getChild("ModuleName") || around.getChild("ModuleNameUnknown")
    )

    if (!tag && !module) return null

    const block = module ? ModuleMap.get(module) : BlockMap.get(tag!)

    if (!block || !block.argumentCompletions) return null

    const options = block.argumentCompletions
    return { from: tree.from, to: pos, options }
  }

  // block node argument values
  else if (
    tree.parent?.name === "BlockNodeArgument" &&
    (tree.name === "BlockNodeArgumentMarkOpen" ||
      tree.name === "BlockNodeArgumentValue") &&
    pos < tree.parent.to
  ) {
    const prop = text(
      findParent(tree, "BlockNodeArgument")?.getChild("BlockNodeArgumentName")
    )

    if (!prop) return null

    const node = findParent(tree, "BlockNode")
    const tag = text(node?.getChild("BlockName"))
    const module = text(
      node?.getChild("ModuleName") || node?.getChild("ModuleNameUnknown")
    )

    if (!tag && !module) return null

    const block = module ? ModuleMap.get(module) : BlockMap.get(tag!)

    const options =
      block?.arguments?.get(prop)?.enumCompletions ??
      htmlEnumCompletions.get(prop) ??
      null

    if (options) {
      return tree.name === "BlockNodeArgumentValue"
        ? { from: tree.from, to: pos, options }
        : { from: tree.from + 1, to: pos, options }
    }
  }

  return null
}

function findParent(tree: SyntaxNode, parent: string | string[]) {
  if (typeof parent === "string") parent = [parent]
  for (let cur = tree.parent; cur; cur = cur.parent) {
    if (parent.includes(cur.name)) return cur
  }
  return null
}
