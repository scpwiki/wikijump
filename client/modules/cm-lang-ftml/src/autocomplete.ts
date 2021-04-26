import { syntaxTree } from "@codemirror/language"
import type { SyntaxNode } from "lezer-tree"
import type {
  Completion,
  CompletionContext,
  CompletionResult
} from "@codemirror/autocomplete"
import type { Block, Attribute } from "./data/types"
import { blocks as blocks2, blockNames } from "./data/blocks"
import { htmlAttributes } from "./data/html-attributes"
import { FTMLFragment } from "ftml-wasm-worker"

const blocksAutocompletion: Completion[] = Object.entries(blocks2).map(
  ([alias, block]) => {
    if (block.info) {
      const infoFragment = new FTMLFragment(block.info)
      return {
        label: alias,
        detail: block.name,
        type: "type",
        async info() {
          return (await infoFragment.render()).fragment
        }
      }
    } else {
      return {
        label: alias,
        detail: block.name,
        type: "type"
      }
    }
  }
)

export function completeFTML(context: CompletionContext): CompletionResult | null {
  const { state, pos } = context
  const around = syntaxTree(state).resolve(pos)
  const tree = around.resolve(pos, -1)

  const text = (tree: SyntaxNode | null | undefined) => {
    if (!tree) return null
    return context.state.sliceDoc(tree.from, tree.to)
  }

  // Tag names
  if (tree.name === "BlockNameUnknown" || tree.name === "BlockName") {
    return { from: tree.from, to: pos, options: blocksAutocompletion }
  }
  // Module names
  // else if (tree.name === "ModuleName") {
  //   return { from: tree.from, to: pos, options: moduleAutoComplete }
  // }
  // Block node attribute names
  else if (tree.name === "BlockLabel") {
    const name = text(around.getChild("BlockName"))
    const module = text(around.getChild("ModuleName"))

    const attributes = new Set<string>()

    if (name && blocks2?.[name]) {
      const block = blocks2[name]!
      const { attrs, globals } = block
      for (const attr of attrs) {
        attributes.add(attr.name)
      }
      if (globals) {
        for (const attr of htmlAttributes) {
          attributes.add(attr.name)
        }
      }
    }

    const options: Completion[] = []

    attributes.forEach(attr =>
      options.push({
        label: attr,
        apply: `${attr}=""`,
        type: "property"
      })
    )

    console.log(options)

    return { from: tree.from, to: pos, options }
  }
  // Block node attribute values
  else if (tree.parent?.name === "BlockNodeParameterValue") {
    const prop = text(findParent(tree, "BlockNodeParameter")?.getChild("propertyName"))
    const node = findParent(tree, ["BlockNode", "BlockContainerNode"])
    const tag = text(node?.getChild("BlockName"))
    const module = text(node?.getChild("ModuleName"))
    let values: string[] | null = null

    // // tag completion
    // if (prop && tag && blocks?.[tag]?.attrs?.[prop]) {
    //   values = blocks[tag]!.attrs![prop]
    // }

    // // module completion
    // else if (prop && module && modules?.[module]?.attrs?.[prop]) {
    //   values = modules[module]!.attrs![prop]!
    // }

    // if (values) {
    //   return {
    //     from: tree.name === "string" ? tree.from + 1 : tree.from,
    //     to: pos,
    //     options: values.map(value => ({
    //       label: value,
    //       type: "enum"
    //     }))
    //   }
    // }
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
