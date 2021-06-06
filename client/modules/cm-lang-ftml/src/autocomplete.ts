import type {
  Completion,
  CompletionContext,
  CompletionResult
} from "@codemirror/autocomplete"
import { syntaxTree } from "@codemirror/language"
import type { SyntaxNode } from "lezer-tree"
import { EditorSvelteComponent } from "sheaf-core"
import { Prism } from "wj-prism"
import { blocks, blockTips, modules } from "./data/blocks"
import { htmlAttributes } from "./data/html-attributes"
import type { Block, Module } from "./data/types"
import ModuleTip from "./tips/ModuleTip.svelte"
import { aliasesFiltered } from "./util"

const blocksAutocompletion: Completion[] = Object.entries(blocks).flatMap(
  ([name, block]) => {
    const aliases = aliasesFiltered([name, block])
    const completions: Completion[] = aliases.map(alias => ({
      label: alias,
      type: "type",
      info: () => blockTips[alias].dom
    }))
    return completions
  }
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

const moduleAutocompletion: Completion[] = Object.entries(modules).flatMap(
  ([name, module]) => {
    const aliases = aliasesFiltered([name, module])
    const handler = new EditorSvelteComponent(ModuleTip)
    const instance = handler.create(undefined, { pass: { name, module } })
    const completions: Completion[] = aliases.map(alias => ({
      label: alias,
      type: "class",
      info: () => instance.dom
    }))
    return completions
  }
)

const htmlAutoCompletion: Completion[] = Object.entries(htmlAttributes).map(
  ([name, attr]) => {
    // TODO: svelte component for arguments
    return {
      label: name,
      detail: "html",
      type: "property",
      apply: `${name}=""`,
      boost: -1
    }
  }
)

const data: Record<string, Block | Module> = { ...blocks }

for (const name in modules) {
  data[`module_${name}`] = modules[name]
}

const argumentAutocompletion: Record<string, Completion[]> = {}
for (const [name, block] of Object.entries(data)) {
  if (!block.arguments && !block["html-attributes"]) continue
  const aliases = aliasesFiltered([name, block])

  const completions: Completion[] = []

  for (const name in block.arguments) {
    // TODO: svelte component for arguments
    const argument = block.arguments[name]
    completions.push({
      label: name,
      type: "property",
      apply: `${name}=""`
    })
  }

  if (block["html-attributes"]) {
    completions.push(...htmlAutoCompletion)
  }

  for (const alias of aliases) {
    argumentAutocompletion[alias] = completions
  }
}

// add a fake block for "_html"
// we use this later for enum autocompletion
data["_html"] = {
  body: "none",
  arguments: htmlAttributes
}

const enumAutocompletion: Record<string, Record<string, Completion[]>> = {}
for (const [name, block] of Object.entries(data)) {
  if (!block.arguments) continue
  const aliases = aliasesFiltered([name, block])

  const completions: Record<string, Completion[]> = {}

  let empty = true

  for (const name in block.arguments) {
    const argument = block.arguments[name]
    if (!argument.enum && argument.type === "bool") {
      empty = false
      completions[name] = [
        { label: "true", type: "keyword" },
        { label: "false", type: "keyword" }
      ]
      // set one of the two to default if there is a default
      if (argument.default !== undefined) {
        completions[name][Boolean(argument.default) ? 0 : 1].detail = "default"
      }
    } else if (argument.enum) {
      empty = false
      completions[name] = argument.enum.map(_enum => ({
        label: String(_enum),
        type: "enum",
        // mark the value with "default" if it is the default
        detail: argument.default === _enum ? "default" : undefined
      }))
    }
  }

  if (!empty) {
    for (const alias of aliases) {
      enumAutocompletion[alias] = completions
    }
  }
}

// kind of a hack, but what this does is make it so that the
// code block's `type` argument displays the languages Prism
// has available to highlight.
// the issue with that is that Prism doesn't synchronously load
// the languages, so we would just get a list of no languages
// if we tried this normally.
// so we can use a getter, and on the fly figure out what
// languages can be highlighted. this is slower, but
// the getter won't be called very often so it should be fine.
try {
  enumAutocompletion.code = {
    ...enumAutocompletion.code,
    get type() {
      const completions: Completion[] = []
      for (const name of Object.keys(Prism.languages)) {
        if (typeof Prism.languages[name] === "function") continue
        completions.push({
          label: name,
          type: "enum"
        })
      }
      return completions
    }
  }
} catch {}

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

    const name = module ? `module_${module}` : tag

    if (name && name in argumentAutocompletion) {
      const options = argumentAutocompletion[name]
      return { from: tree.from, to: pos, options }
    }
  }

  // block node argument values
  else if (tree.parent?.name === "BlockNodeArgument") {
    const prop = text(
      findParent(tree, "BlockNodeArgument")?.getChild("BlockNodeArgumentName")
    )
    const node = findParent(tree, "BlockNode")
    const tag = text(node?.getChild("BlockName"))
    const module = text(
      node?.getChild("ModuleName") || node?.getChild("ModuleNameUnknown")
    )

    let name = module ? `module_${module}` : tag

    if (name && prop) {
      // check for html attributes
      if (data?.[name]?.["html-attributes"] && enumAutocompletion?.["_html"]?.[prop]) {
        name = "_html"
      }

      if (enumAutocompletion?.[name]?.[prop]) {
        // check if were past the last quote mark or not
        if (pos >= tree.parent.to) return null
        // ensure that we're inbetween the quotes
        if (
          tree.name === "BlockNodeArgumentMarkOpen" ||
          tree.name === "BlockNodeArgumentValue"
        ) {
          const options = enumAutocompletion[name][prop]
          // offset pos one position if we're at the first quote mark
          return tree.name === "BlockNodeArgumentValue"
            ? { from: tree.from, to: pos, options }
            : { from: tree.from + 1, to: pos, options }
        }
      }
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
