import { syntaxTree } from "@codemirror/language"
import type { SyntaxNode } from "lezer-tree"
import type {
  Completion,
  CompletionContext,
  CompletionResult
} from "@codemirror/autocomplete"

interface TagSpec {
  info?: string
  attrs?: Record<string, null | string[]>
  keywords?: string[]
}

type BlockTypes = "block" | "module"

const tagAttrs: Record<string, null | string[]> = {
  id: null,
  class: null,
  style: null
}

const tag: TagSpec = {
  attrs: tagAttrs
}

// TODO: fully match FTML
const blockData: Record<BlockTypes, Record<string, TagSpec | null>> = {
  block: {
    // generic blocks
    "image": null,
    "rate": null,
    "toc": null,
    "footnoteblock": null,
    "button": {
      attrs: {
        text: null,
        class: null,
        style: null
      },
      keywords: [
        "set-tags",
        "edit",
        "edit-append",
        "edit-sections",
        "history",
        "print",
        "files",
        "tags",
        "source",
        "backlinks",
        "talk",
        "delete",
        "rename",
        "site-tools",
        "edit-meta",
        "watchers",
        "parent",
        "lock-page"
      ]
    },
    // text
    "date": null,
    "user": null,
    "social": null,
    "#": null,
    // expr
    "#expr": null,
    "#ifexpr": null,
    "#if": null,
    "if": null,
    "else": null,
    // tags
    "span": tag,
    "a": tag,
    "div": tag,
    "ul": tag,
    "ol": tag,
    "li": tag,
    "table": tag,
    "row": tag,
    "column": tag,
    "hcell": tag,
    "cell": tag,
    // align
    "=": null,
    ">": null,
    "<": null,
    "==": null,
    // special
    "module": null,
    "include": null,
    // containers
    "footnote": null,
    "note": null,
    "tabview": null,
    "tab": null,
    "bibliography": null,
    "iftags": null,
    "head": null,
    "body": null,
    "foot": null,
    "collapsible": {
      attrs: {
        show: null,
        hide: null,
        hideLocation: ["both", "top", "bottom"],
        folded: ["yes", "no"]
      }
    },
    // nested
    "html": null,
    "embed": null,
    "embedvideo": null,
    "embedaudio": null,
    "form": null,
    "code": {
      attrs: {
        type: null
      }
    },
    "math": {
      attrs: {
        type: [
          "align",
          "alignat",
          "aligned",
          "alingedat",
          "array",
          "Bmatrix",
          "bmatrix",
          "cases",
          "eqnarray",
          "equation",
          "gather",
          "gathered",
          "matrix",
          "multiline",
          "pmatrix",
          "smallmatrix",
          "split",
          "subarray",
          "Vmatrix",
          "vmatrix"
        ]
      }
    }
  },
  module: {
    css: null,
    listPages: {
      attrs: {
        pagetype: ["normal", "hidden", "*"],
        category: [".", "*", "%%category%%"],
        tags: ["-", "=", "=="],
        parent: ["-", "=", "-=", "."],
        link_to: ["."],
        created_at: ["="],
        updated_at: null,
        created_by: ["=", "-="],
        rating: ["="],
        votes: ["="],
        offset: null,
        range: [".", "before", "after", "others"],
        name: ["="],
        fullname: null,
        order: null,
        limit: null,
        perPage: null,
        reverse: ["yes", "no"],
        separate: ["yes", "no"],
        wrapper: ["yes", "no"],
        header: null,
        footer: null,
        rss: null,
        rssDescription: null,
        rssHome: null,
        rssLimit: null,
        rssOnly: null
      }
    }
  }
}

const blocks: Record<string, TagSpec> = {}
const blockAutocomplete: Completion[] = []
const modules = blockData.module
const moduleAutoComplete: Completion[] = []

// TODO: this doesn't need to be like this anymore
for (const group in blockData) {
  const specs = blockData[group as BlockTypes] as Record<string, TagSpec>
  if (group !== "module") {
    Object.assign(blocks, specs)
    blockAutocomplete.push(
      ...Object.keys(specs).map(
        (name): Completion => ({
          label: name,
          info: specs[name]?.info,
          type: "type"
        })
      )
    )
  } else {
    moduleAutoComplete.push(
      ...Object.keys(specs).map(
        (name): Completion => ({
          label: name,
          info: specs[name]?.info,
          type: "class"
        })
      )
    )
  }
}

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
    return { from: tree.from, to: pos, options: blockAutocomplete }
  }
  // Module names
  else if (tree.name === "ModuleName") {
    return { from: tree.from, to: pos, options: moduleAutoComplete }
  }
  // Block node attribute names
  else if (tree.name === "BlockLabel") {
    const tag = text(around.getChild("BlockName"))
    const module = text(around.getChild("ModuleName"))

    const attrs: string[] = []
    const keywords: string[] = []

    // attributes

    if (tag && blocks?.[tag]?.attrs) {
      attrs.push(...Object.keys(blocks[tag]!.attrs!))
    } else if (module && modules?.[module]?.attrs) {
      attrs.push(...Object.keys(modules[module]!.attrs!))
    }

    // keywords

    if (tag && blocks?.[tag]?.keywords) {
      keywords.push(...blocks[tag]!.keywords!)
    }

    if (module && modules?.[module]?.keywords) {
      keywords.push(...modules[module]!.keywords!)
    }

    const options: Completion[] = attrs
      .map(attr => ({
        label: attr,
        apply: `${attr}=""`,
        type: "property"
      }))
      .concat(
        keywords.map(keyword => ({
          label: keyword,
          apply: keyword,
          type: "keyword"
        }))
      )

    return { from: tree.from, to: pos, options }
  }
  // Block node attribute values
  else if (tree.parent?.name === "BlockNodeParameterValue") {
    const prop = text(findParent(tree, "BlockNodeParameter")?.getChild("propertyName"))
    const node = findParent(tree, ["BlockNode", "BlockContainerNode"])
    const tag = text(node?.getChild("BlockName"))
    const module = text(node?.getChild("ModuleName"))
    let values: string[] | null = null

    // tag completion
    if (prop && tag && blocks?.[tag]?.attrs?.[prop]) {
      values = blocks[tag]!.attrs![prop]
    }

    // module completion
    else if (prop && module && modules?.[module]?.attrs?.[prop]) {
      values = modules[module]!.attrs![prop]!
    }

    if (values) {
      return {
        from: tree.name === "string" ? tree.from + 1 : tree.from,
        to: pos,
        options: values.map(value => ({
          label: value,
          type: "enum"
        }))
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
