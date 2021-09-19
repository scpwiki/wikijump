import { TarnationLanguage } from "@wikijump/cm-tarnation"
import printTreeGrammar from "./print-tree.yaml"

// export const FTMLTokensGrammar = new TarnationLanguage({
//   name: "FTMLTokens",
//   // prettier-ignore
//   grammar: {
//     fallback: ["t.string"],
//     brackets: [
//       { name: "t.squareBracket", pair: ["[", "]"] }
//     ],
//     states: {
//       root: [
//         [/^(\[)(\d+)( <-> )(\d+)(\])(:)(\s*)(\S+)(\s*)(=>)(.*$)/,
//           [
//             "@BR", "t.integer", "t.operator", "t.integer", "@BR", "t.separator", "",
//             "t.content", "", "t.operator", "t.string"
//           ]
//         ]
//       ]
//     }
//   }
// })

export const PrintTreeGrammar = new TarnationLanguage({
  name: "PrintTree",
  grammar: printTreeGrammar as any
})
