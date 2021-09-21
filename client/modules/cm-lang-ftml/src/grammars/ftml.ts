import { TarnationLanguage } from "@wikijump/cm-tarnation"
import { addLanguages, languageList } from "@wikijump/codemirror"
import { cssCompletion, htmlCompletion } from "@wikijump/codemirror/cm"
import { completeFTML } from "../autocomplete"
import { blocks, modules } from "../data/blocks"
import { ftmlHoverTooltips } from "../hover"
import { ftmlLinter } from "../lint"
import { spellcheckFTML } from "../spellcheck"
import { aliasesFiltered, aliasesRaw } from "../util"
import { StyleAttributeGrammar } from "./css-attributes"
import ftmlGrammar from "./ftml.yaml"
import { TexLanguage } from "./tex"

const blockEntries = Object.entries(blocks)
const moduleEntries = Object.entries(modules)

export const FTMLLanguage = new TarnationLanguage({
  name: "FTML",

  nestLanguages: languageList,

  languageData: {
    autocomplete: completeFTML,
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
            return { name: value, overlay: [{ from: cursor.from, to: cursor.to }] }
          }
        }
      }

      return null
    }
  },

  grammar: ftmlGrammar as any
})
