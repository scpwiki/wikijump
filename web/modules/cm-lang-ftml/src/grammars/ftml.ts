import type { SyntaxNode } from "@lezer/common"
import { addLanguages, languageList } from "@wikijump/codemirror"
import { cssCompletion, htmlCompletion, type Completion } from "@wikijump/codemirror/cm"
import type { TarnationCompletionContext } from "cm-tarnation"
import { TarnationLanguage } from "cm-tarnation"
import { BlockMap, Blocks, BlockSet, ModuleMap, Modules, ModuleSet } from "../data/data"
import { htmlEnumCompletions } from "../data/html-attributes"
import { ftmlHoverTooltips } from "../hover"
import { ftmlLinter } from "../lint"
import { spellcheckFTML } from "../spellcheck"
import { aliasesFiltered, aliasesRaw } from "../util"
import { StyleAttributeGrammar } from "./css-attributes"
import ftmlGrammar from "./ftml.yaml"
import { TexLanguage } from "./tex"

const blockEntries = Object.entries(Blocks)
const moduleEntries = Object.entries(Modules)

const blockCompletions: Completion[] = Array.from(BlockSet).flatMap(
  block => block.completions
)

// we're also going to push special completions for "module" and "include"
blockCompletions.push(
  {
    label: "module",
    type: "keyword"
  },
  {
    label: "include-elements",
    type: "keyword"
  },
  {
    label: "include-messy",
    type: "keyword"
  }
)

const moduleCompletions: Completion[] = Array.from(ModuleSet).flatMap(
  module => module.completions
)

export const FTMLLanguage = new TarnationLanguage({
  name: "FTML",

  nestLanguages: languageList,

  languageData: {
    spellcheck: spellcheckFTML
  },

  supportExtensions: [
    ftmlLinter,
    ftmlHoverTooltips,
    htmlCompletion,
    cssCompletion,
    addLanguages(TexLanguage.description, StyleAttributeGrammar.description)
  ],

  configure: {
    variables: {
      blk_map: blockEntries
        .filter(([, { head, body }]) => head === "map" && body === "none")
        .flatMap(aliasesFiltered),

      blk_val: blockEntries
        .filter(([, { head, body }]) => head === "value" && body === "none")
        .flatMap(aliasesFiltered),

      blk_valmap: blockEntries
        .filter(([, { head, body }]) => head === "value+map" && body === "none")
        .flatMap(aliasesFiltered),

      blk_el: blockEntries
        .filter(([, { head, body }]) => head === "none" && body === "elements")
        .flatMap(aliasesFiltered),

      blk_map_el: blockEntries
        .filter(([, { head, body }]) => head === "map" && body === "elements")
        .flatMap(aliasesFiltered),

      blk_val_el: blockEntries
        .filter(([, { head, body }]) => head === "value" && body === "elements")
        .flatMap(aliasesFiltered),

      // currently empty
      // blk_valmap_el: blockEntries
      //   .filter(([, { head, body }]) => head === "value+map" && body === "elements")
      //   .flatMap(aliasesFiltered),

      mods: moduleEntries.flatMap(aliasesRaw),

      blk_align: ["=", "==", "<", ">"]
    },

    // nesting function so that `[[code type="foo"]]` nests languages
    nest(cursor, input) {
      if (cursor.type.name === "BlockNestedCodeInside") {
        // find the starting blocknode
        const startNode = cursor.node.parent?.firstChild
        if (!startNode) return null

        // check its arguments
        for (const arg of startNode.getChildren("BlockNodeArgument")) {
          const nameNode = arg.getChild("BlockNodeArgumentName")
          if (!nameNode) continue
          // check argument name, then check argument value
          if (input.read(nameNode.from, nameNode.to).toLowerCase() === "type") {
            const valueNode = arg.getChild("BlockNodeArgumentValue")
            if (!valueNode) continue
            const value = input.read(valueNode.from, valueNode.to)
            return { name: value }
          }
        }
      }

      return null
    },

    autocomplete: {
      _alsoEmitNames: true,
      _alsoTypeNames: true,
      _traverseUpwards: true,

      "BlockNameUnknown BlockName": ctx => ({
        from: ctx.from,
        to: ctx.to,
        options: blockCompletions
      }),

      "ModuleNameUnknown ModuleName": ctx => ({
        from: ctx.from,
        to: ctx.to,
        options: moduleCompletions
      }),

      // just typed a new block (`[[]]`)
      "BlockCompletelyEmpty": ctx => {
        // must be [[_]] (underscore being the cursor)
        if (ctx.pos !== ctx.from + 2) return null
        return { from: ctx.pos, to: ctx.pos, options: blockCompletions }
      },

      // incomplete block node arguments,
      "BlockLabel": ctx => {
        const block = blockOf(ctx, ctx.around)
        if (!block || !block.argumentCompletions) return null
        return { from: ctx.from, to: ctx.to, options: block.argumentCompletions }
      },

      // explicit request for argument completions
      "BlockNode": ctx => {
        if (ctx.traversed || !ctx.explicit) return null
        const block = blockOf(ctx, ctx.node)
        if (!block || !block.argumentCompletions) return null
        return { from: ctx.pos, to: ctx.pos, options: block.argumentCompletions }
      },

      // block node argument values
      "BlockNodeArgumentValue BlockNodeArgumentMarkOpen": ctx => {
        const argument = ctx.parent("BlockNodeArgument")
        const prop = ctx.textOf(argument?.getChild("BlockNodeArgumentName"))
        if (!prop) return null

        const node = ctx.parent("BlockNode", 2)
        const block = blockOf(ctx, node)
        if (!block) return null

        let options = block.arguments?.get(prop)?.enumCompletions
        if (!options && block.htmlAttributes) options = htmlEnumCompletions.get(prop)

        if (options) {
          return ctx.type.name === "BlockNodeArgumentValue"
            ? { from: ctx.from, to: ctx.to, options }
            : { from: ctx.from + 1, to: ctx.to, options }
        }

        return null
      }
    }
  },

  grammar: ftmlGrammar as any
})

function blockOf(ctx: TarnationCompletionContext, node: SyntaxNode | null | undefined) {
  if (!node) return null
  const tag = ctx.textOf(node?.getChild("BlockName"))
  const module = ctx.textOf(
    node?.getChild("ModuleName") || node?.getChild("ModuleNameUnknown")
  )
  if (!tag && !module) return null
  const block = module ? ModuleMap.get(module) : BlockMap.get(tag!)
  return block ?? null
}
