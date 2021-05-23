import { tags as t } from "@codemirror/highlight"
import { TarnationLanguage } from "cm-tarnation"
import type { Grammar } from "cm-tarnation/src/grammar/definition"

export const TexLanguage = new TarnationLanguage({
  name: "wikimath",

  languageData: {
    commentTokens: { line: "%" }
  },

  // prettier-ignore
  grammar: (): Grammar => ({
    start: "root",

    variables: {
      texBrackets: ["(", ")", "[", "]", "{", "}"],
      texSymbols: ["+", "-", "=", "!", "/", "<", ">", "|", "'", ":", "*", "^"]
    },

    brackets: [
      { name: "t.paren", pair: ["(", ")"] },
      { name: "t.brace", pair: ["{", "}"] },
      { name: "t.squareBracket", pair: ["[", "]"] }
    ],

    fallback: ["t.emphasis"],

    states: {
      root: [
        [/%.*$/, "t.comment"], // %comments

        { style: {
          Function: t.function(t.name),
          Command: t.string
        }},

        [/([a-zA-Z]+)(?=\([^]*?\))/, "Function"], // styles 'fn()'

        [/(\\#?[a-zA-Z\d]+)(\{)([^]*?)(\})/, "CommandGroup",
          ["Command", "@BR", "t.string", "@BR"]
        ],

        [/\\#?[a-zA-Z\d]+/, "Command"],

        [/\\[,>;!]/, "t.string"],       // spacing
        [/\\+/, "t.escape"],            // \\ and the like
        [/\^(?!\d)|[_&]/, "t.keyword"], // special keywords/operators

        [/\d+/, "t.unit"],             // numbers
        [/@texSymbols/, "t.operator"], // operators
        [/@texBrackets/, "@BR"]        // brackets
      ]
    }
  })
})
